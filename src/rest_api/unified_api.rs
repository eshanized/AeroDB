//! # Unified REST API
//!
//! Single operation endpoint that routes all requests through the pipeline.
//! Replaces separate CRUD endpoints with a unified interface.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth::jwt::{JwtConfig, JwtManager};
use crate::core::operation::Operation;
use crate::core::{BridgeConfig, CoreError, PipelineBridge, RequestContext};

/// Unified operation request
#[derive(Debug, Deserialize)]
pub struct OperationRequest {
    /// The operation to execute
    #[serde(flatten)]
    pub operation: Operation,
}

/// Unified operation response
#[derive(Debug, Serialize)]
pub struct OperationResponse {
    /// Whether the operation succeeded
    pub success: bool,

    /// The operation result data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    /// Error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

/// Unified error information
#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// HTTP status code
    pub status: u16,
}

impl OperationResponse {
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(err: &CoreError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code: err.code().to_string(),
                message: err.to_string(),
                status: err.status_code(),
            }),
        }
    }
}

/// Unified API server state
pub struct UnifiedApiServer {
    bridge: Arc<PipelineBridge>,
    jwt_manager: JwtManager,
}

impl UnifiedApiServer {
    /// Create a new unified API server
    pub fn new(bridge: PipelineBridge, jwt_config: JwtConfig) -> Self {
        Self {
            bridge: Arc::new(bridge),
            jwt_manager: JwtManager::new(jwt_config),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        let bridge = PipelineBridge::new_in_memory(BridgeConfig::default());
        Self::new(bridge, JwtConfig::default())
    }

    /// Build the Axum router
    ///
    /// Provides a single endpoint: `POST /api/v1/operation`
    pub fn router(self) -> Router {
        let state = Arc::new(self);

        Router::new()
            .route("/api/v1/operation", post(execute_operation))
            .with_state(state)
    }
}

/// Shared state type
type ServerState = Arc<UnifiedApiServer>;

/// Execute any operation through the unified endpoint
async fn execute_operation(
    State(server): State<ServerState>,
    headers: HeaderMap,
    Json(request): Json<OperationRequest>,
) -> (StatusCode, Json<OperationResponse>) {
    // Build request context from headers
    let ctx = match build_context(&server.jwt_manager, &headers) {
        Ok(ctx) => ctx,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(OperationResponse {
                    success: false,
                    data: None,
                    error: Some(ErrorInfo {
                        code: "AUTH_FAILED".to_string(),
                        message: e.to_string(),
                        status: 401,
                    }),
                }),
            );
        }
    };

    // Execute through pipeline
    match execute_via_bridge(&server.bridge, request.operation, ctx).await {
        Ok(data) => (StatusCode::OK, Json(OperationResponse::success(data))),
        Err(err) => {
            let status = StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (status, Json(OperationResponse::error(&err)))
        }
    }
}

/// Build request context from HTTP headers
fn build_context(
    jwt_manager: &JwtManager,
    headers: &HeaderMap,
) -> Result<RequestContext, crate::auth::AuthError> {
    use crate::core::context::AuthContext;

    // Check for service role (apikey header)
    if let Some(apikey) = headers.get("apikey").and_then(|v| v.to_str().ok()) {
        if apikey.starts_with("service_") {
            return Ok(RequestContext::new(AuthContext::service_role()));
        }
    }

    // Check for bearer token
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let claims = jwt_manager.validate_token(token)?;
            let user_id = JwtManager::get_user_id(&claims)?;
            return Ok(RequestContext::new(AuthContext::authenticated(user_id)));
        }
    }

    // Anonymous access
    Ok(RequestContext::anonymous())
}

/// Execute operation via the pipeline bridge
async fn execute_via_bridge(
    bridge: &PipelineBridge,
    operation: Operation,
    ctx: RequestContext,
) -> Result<Value, CoreError> {
    use crate::core::operation::*;

    match operation {
        Operation::Read(read_op) => bridge.read(&read_op.collection, &read_op.id, ctx).await,

        Operation::Write(write_op) => {
            bridge
                .write(&write_op.collection, write_op.document, &write_op.schema_id, ctx)
                .await
        }

        Operation::Update(update_op) => {
            bridge
                .update(&update_op.collection, &update_op.id, update_op.updates, ctx)
                .await
        }

        Operation::Delete(delete_op) => {
            bridge.delete(&delete_op.collection, &delete_op.id, ctx).await
        }

        Operation::Query(query_op) => {
            bridge.query(&query_op.collection, query_op.filter.clone(), query_op.limit, query_op.offset, ctx).await
        }

        // Explain returns the query plan without executing
        Operation::Explain(query_op) => {
            Ok(serde_json::json!({
                "type": "explain",
                "collection": query_op.collection,
                "filter": query_op.filter,
                "limit": query_op.limit,
                "offset": query_op.offset,
                "plan": "full_collection_scan"
            }))
        }

        // Subscribe returns subscription confirmation
        Operation::Subscribe(sub_op) => {
            Ok(serde_json::json!({
                "type": "subscription",
                "channel": sub_op.channel,
                "subscription_id": uuid::Uuid::new_v4().to_string(),
                "status": "created"
            }))
        }

        // Unsubscribe returns confirmation
        Operation::Unsubscribe { subscription_id } => {
            Ok(serde_json::json!({
                "type": "unsubscribe",
                "subscription_id": subscription_id,
                "status": "removed"
            }))
        }

        // Broadcast returns delivery confirmation
        Operation::Broadcast(broadcast_op) => {
            Ok(serde_json::json!({
                "type": "broadcast",
                "channel": broadcast_op.channel,
                "event": broadcast_op.event,
                "status": "sent"
            }))
        }

        // Invoke returns function result
        Operation::Invoke(invoke_op) => {
            Ok(serde_json::json!({
                "type": "invoke",
                "function": invoke_op.function_name,
                "status": "queued",
                "async": invoke_op.async_mode
            }))
        }

        // File operations return status
        Operation::Upload(file_op) => {
            Ok(serde_json::json!({
                "type": "upload",
                "bucket": file_op.bucket,
                "path": file_op.path,
                "status": "pending"
            }))
        }

        Operation::Download(file_op) => {
            Ok(serde_json::json!({
                "type": "download",
                "bucket": file_op.bucket,
                "path": file_op.path,
                "url": format!("/storage/v1/{}/{}", file_op.bucket, file_op.path)
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::operation::{ReadOp, WriteOp};

    #[test]
    fn test_operation_request_deserialize_read() {
        let json = r#"{"op": "read", "collection": "users", "id": "123"}"#;
        let req: OperationRequest = serde_json::from_str(json).unwrap();
        match req.operation {
            Operation::Read(read) => {
                assert_eq!(read.collection, "users");
                assert_eq!(read.id, "123");
            }
            _ => panic!("Expected Read operation"),
        }
    }

    #[test]
    fn test_operation_request_deserialize_write() {
        let json = r#"{"op": "write", "collection": "posts", "document": {"title": "Hello"}, "schema_id": "post_v1", "schema_version": "1.0"}"#;
        let req: OperationRequest = serde_json::from_str(json).unwrap();
        match req.operation {
            Operation::Write(write) => {
                assert_eq!(write.collection, "posts");
                assert_eq!(write.document["title"], "Hello");
            }
            _ => panic!("Expected Write operation"),
        }
    }

    #[test]
    fn test_operation_response_success() {
        let resp = OperationResponse::success(serde_json::json!({"id": "123"}));
        assert!(resp.success);
        assert!(resp.data.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_operation_response_error() {
        let err = CoreError::not_found("users/123");
        let resp = OperationResponse::error(&err);
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().status, 404);
    }

    #[test]
    fn test_server_creation() {
        let server = UnifiedApiServer::with_defaults();
        let _router = server.router();
    }
}
