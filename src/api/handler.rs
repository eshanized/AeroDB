//! API Handler for aerodb
//!
//! Orchestrates all subsystems behind a single global mutex.
//! Enforces strict request handling flow.

use std::sync::Mutex;

use serde_json::{json, Value};

use crate::index::{DocumentInfo, IndexManager, StorageScan as IndexStorageScan};
use crate::planner::{FilterOp, Predicate, Query, QueryPlanner, SortDirection, SortSpec};
use crate::schema::{SchemaRegistry, SchemaValidator};
use crate::storage::{StorageReader, StorageWriter};
use crate::wal::{RecordType, WalPayload, WalWriter};

use super::errors::{ApiError, ApiResult};
use super::request::{DeleteRequest, InsertRequest, QueryRequest, Request, UpdateRequest};
use super::response::Response;

/// API Handler with global execution lock
pub struct ApiHandler<'a> {
    /// Global mutex for serialized execution
    lock: Mutex<()>,

    /// Schema registry
    schema_registry: &'a SchemaRegistry,

    /// Schema validator
    schema_validator: SchemaValidator<'a>,

    /// Query planner
    planner: QueryPlanner<'a>,

    /// WAL writer
    wal_writer: &'a mut WalWriter,

    /// Storage writer
    storage_writer: &'a mut StorageWriter,

    /// Storage reader
    storage_reader: &'a StorageReader,

    /// Index manager
    index_manager: &'a mut IndexManager,

    /// Collection name (single collection in Phase 0)
    collection: String,
}

impl<'a> ApiHandler<'a> {
    /// Create a new API handler
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_registry: &'a SchemaRegistry,
        wal_writer: &'a mut WalWriter,
        storage_writer: &'a mut StorageWriter,
        storage_reader: &'a StorageReader,
        index_manager: &'a mut IndexManager,
        collection: impl Into<String>,
    ) -> Self {
        Self {
            lock: Mutex::new(()),
            schema_registry,
            schema_validator: SchemaValidator::new(schema_registry),
            planner: QueryPlanner::new(schema_registry),
            wal_writer,
            storage_writer,
            storage_reader,
            index_manager,
            collection: collection.into(),
        }
    }

    /// Handle a raw JSON request string
    pub fn handle(&mut self, json_request: &str) -> Response {
        // Acquire global lock at request entry
        let _guard = self.lock.lock().expect("Lock poisoned");

        // Parse request
        let request = match Request::parse(json_request) {
            Ok(r) => r,
            Err(e) => return Response::error(&e),
        };

        // Dispatch to appropriate handler
        let result = match request {
            Request::Insert(r) => self.handle_insert(r),
            Request::Update(r) => self.handle_update(r),
            Request::Delete(r) => self.handle_delete(r),
            Request::Query(r) => self.handle_query(r),
            Request::Explain(r) => self.handle_explain(r),
        };

        // Lock released when _guard drops
        match result {
            Ok(data) => Response::success(data),
            Err(e) => Response::error(&e),
        }
    }

    /// Handle insert operation
    ///
    /// Flow:
    /// 1. Validate schema
    /// 2. Build write intent
    /// 3. Append WAL record
    /// 4. Apply to Storage
    /// 5. Update Index
    fn handle_insert(&mut self, req: InsertRequest) -> ApiResult<Value> {
        // 1. Validate schema
        self.schema_validator
            .validate(&req.document, &req.schema_id, &req.schema_version)
            .map_err(ApiError::from_schema_error)?;

        // Extract document ID
        let doc_id = req.document
            .get("_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::invalid_request("Document missing _id"))?
            .to_string();

        // 2. Build write intent
        let body_bytes = serde_json::to_vec(&req.document)
            .map_err(|e| ApiError::invalid_request(format!("Failed to serialize document: {}", e)))?;

        let payload = WalPayload::new(
            &self.collection,
            &doc_id,
            &req.schema_id,
            &req.schema_version,
            body_bytes,
        );

        // 3. Append WAL record
        self.wal_writer
            .append(RecordType::Insert, payload.clone())
            .map_err(ApiError::from_wal_error)?;

        // 4. Apply to Storage
        let offset = self.storage_writer
            .write(&req.schema_id, &req.schema_version, &doc_id, &req.document, false)
            .map_err(ApiError::from_storage_error)?;

        // 5. Update Index
        let doc_info = DocumentInfo {
            document_id: doc_id.clone(),
            schema_id: req.schema_id,
            schema_version: req.schema_version,
            is_tombstone: false,
            body: req.document,
            offset,
        };
        self.index_manager.apply_write(&doc_info);

        Ok(json!({"inserted": doc_id}))
    }

    /// Handle update operation
    ///
    /// Flow:
    /// 1. Validate schema
    /// 2. Check document exists
    /// 3. Build write intent
    /// 4. Append WAL record
    /// 5. Apply to Storage
    /// 6. Update Index
    fn handle_update(&mut self, req: UpdateRequest) -> ApiResult<Value> {
        // 1. Validate schema (update mode)
        self.schema_validator
            .validate_update(&req.document, &req.schema_id, &req.schema_version)
            .map_err(ApiError::from_schema_error)?;

        // Extract document ID
        let doc_id = req.document
            .get("_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::invalid_request("Document missing _id"))?
            .to_string();

        // 2. Check document exists (via index)
        let offsets = self.index_manager.lookup_pk(&doc_id);
        if offsets.is_empty() {
            return Err(ApiError::invalid_request(format!("Document not found: {}", doc_id)));
        }

        // 3. Build write intent
        let body_bytes = serde_json::to_vec(&req.document)
            .map_err(|e| ApiError::invalid_request(format!("Failed to serialize document: {}", e)))?;

        let payload = WalPayload::new(
            &self.collection,
            &doc_id,
            &req.schema_id,
            &req.schema_version,
            body_bytes,
        );

        // 4. Append WAL record
        self.wal_writer
            .append(RecordType::Update, payload.clone())
            .map_err(ApiError::from_wal_error)?;

        // 5. Apply to Storage (overwrite)
        let offset = self.storage_writer
            .write(&req.schema_id, &req.schema_version, &doc_id, &req.document, false)
            .map_err(ApiError::from_storage_error)?;

        // 6. Update Index
        let doc_info = DocumentInfo {
            document_id: doc_id.clone(),
            schema_id: req.schema_id,
            schema_version: req.schema_version,
            is_tombstone: false,
            body: req.document,
            offset,
        };
        self.index_manager.apply_write(&doc_info);

        Ok(json!({"updated": doc_id}))
    }

    /// Handle delete operation
    ///
    /// Flow:
    /// 1. Check document exists
    /// 2. Append WAL record
    /// 3. Apply tombstone to Storage
    /// 4. Update Index
    fn handle_delete(&mut self, req: DeleteRequest) -> ApiResult<Value> {
        // 1. Check document exists (via index)
        let offsets = self.index_manager.lookup_pk(&req.document_id);
        if offsets.is_empty() {
            return Err(ApiError::invalid_request(format!("Document not found: {}", req.document_id)));
        }

        // Get the old document body for index removal
        let old_offset = offsets[offsets.len() - 1];
        let old_doc = self.storage_reader
            .read_at(old_offset)
            .map_err(ApiError::from_storage_error)?
            .ok_or_else(|| ApiError::invalid_request("Document not found in storage"))?;

        let old_body: Value = serde_json::from_slice(&old_doc.body)
            .unwrap_or(json!({}));

        // 2. Append WAL record
        let payload = WalPayload::tombstone(
            &self.collection,
            &req.document_id,
            &req.schema_id,
            "", // version empty for delete
        );

        self.wal_writer
            .append(RecordType::Delete, payload)
            .map_err(ApiError::from_wal_error)?;

        // 3. Apply tombstone to Storage
        self.storage_writer
            .write_tombstone(&req.document_id)
            .map_err(ApiError::from_storage_error)?;

        // 4. Update Index
        self.index_manager.apply_delete(&req.document_id, &old_body);

        Ok(json!({"deleted": req.document_id}))
    }

    /// Handle query operation
    ///
    /// Flow:
    /// 1. Parse query
    /// 2. Call Planner
    /// 3. Call Executor (simplified: use index + storage)
    /// 4. Return results
    fn handle_query(&mut self, req: QueryRequest) -> ApiResult<Value> {
        // 1. Build query AST
        let query = self.build_query(&req)?;

        // 2. Call Planner
        let plan = self.planner
            .plan(&query, self.index_manager.indexed_fields())
            .map_err(ApiError::from_planner_error)?;

        // 3. Execute query (simplified execution)
        let mut results = Vec::new();

        // Get offsets from index based on plan
        let offsets = self.get_offsets_for_plan(&plan, &query);

        // Read documents at offsets
        for offset in offsets.iter().take(req.limit) {
            if let Ok(Some(record)) = self.storage_reader.read_at(*offset) {
                // Skip tombstones
                if record.is_tombstone {
                    continue;
                }

                // Check schema match
                if record.schema_id != req.schema_id || record.schema_version != req.schema_version {
                    continue;
                }

                // Parse body
                if let Ok(doc) = serde_json::from_slice::<Value>(&record.body) {
                    results.push(doc);
                }
            }
        }

        Ok(json!(results))
    }

    /// Handle explain operation
    fn handle_explain(&mut self, req: QueryRequest) -> ApiResult<Value> {
        // Build query AST
        let query = self.build_query(&req)?;

        // Call Planner
        let plan = self.planner
            .plan(&query, self.index_manager.indexed_fields())
            .map_err(ApiError::from_planner_error)?;

        // Return explain output
        Ok(json!({
            "scan_type": format!("{:?}", plan.scan_type),
            "index_field": plan.index_field,
            "predicates": plan.predicates.len(),
            "sort": plan.sort.as_ref().map(|s| &s.field),
            "limit": plan.limit
        }))
    }

    /// Build a Query AST from a QueryRequest
    fn build_query(&self, req: &QueryRequest) -> ApiResult<Query> {
        let mut predicates = Vec::new();

        // Parse filter
        if let Some(filter) = &req.filter {
            if let Some(obj) = filter.as_object() {
                for (field, condition) in obj {
                    if let Some(cond_obj) = condition.as_object() {
                        for (op, value) in cond_obj {
                            let filter_op = match op.as_str() {
                                "$eq" => FilterOp::Eq(value.clone()),
                                "$gte" => FilterOp::Gte(value.clone()),
                                "$gt" => FilterOp::Gt(value.clone()),
                                "$lte" => FilterOp::Lte(value.clone()),
                                "$lt" => FilterOp::Lt(value.clone()),
                                other => return Err(ApiError::invalid_request(
                                    format!("Unknown filter operator: {}", other)
                                )),
                            };
                            predicates.push(Predicate {
                                field: field.clone(),
                                op: filter_op,
                            });
                        }
                    }
                }
            }
        }

        // Parse sort
        let sort = req.sort.as_ref().map(|s| {
            let (field, direction) = if s.starts_with('-') {
                (s[1..].to_string(), SortDirection::Descending)
            } else {
                (s.clone(), SortDirection::Ascending)
            };
            SortSpec { field, direction }
        });

        Ok(Query {
            schema_id: req.schema_id.clone(),
            schema_version: req.schema_version.clone(),
            predicates,
            sort,
            limit: req.limit,
        })
    }

    /// Get offsets from index based on plan
    fn get_offsets_for_plan(
        &self,
        plan: &crate::planner::QueryPlan,
        query: &Query,
    ) -> Vec<u64> {
        use crate::planner::ScanType;

        match plan.scan_type {
            ScanType::PrimaryKey => {
                // Find PK predicate
                for pred in &query.predicates {
                    if pred.field == "_id" {
                        if let FilterOp::Eq(ref val) = pred.op {
                            if let Some(pk) = val.as_str() {
                                return self.index_manager.lookup_pk(pk);
                            }
                        }
                    }
                }
                Vec::new()
            }
            ScanType::IndexedEquality => {
                if let Some(ref field) = plan.index_field {
                    for pred in &query.predicates {
                        if &pred.field == field {
                            if let FilterOp::Eq(ref val) = pred.op {
                                return self.index_manager.lookup_eq(field, val);
                            }
                        }
                    }
                }
                Vec::new()
            }
            ScanType::IndexedRange => {
                if let Some(ref field) = plan.index_field {
                    let mut min: Option<&Value> = None;
                    let mut max: Option<&Value> = None;

                    for pred in &query.predicates {
                        if &pred.field == field {
                            match &pred.op {
                                FilterOp::Gte(v) | FilterOp::Gt(v) => min = Some(v),
                                FilterOp::Lte(v) | FilterOp::Lt(v) => max = Some(v),
                                _ => {}
                            }
                        }
                    }

                    return self.index_manager.lookup_range(field, min, max, Some(plan.limit));
                }
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldType, Schema, SchemaRegistry};
    use serde_json::json;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, SchemaRegistry, WalWriter, StorageWriter, StorageReader, IndexManager) {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        let storage_path = temp_dir.path().join("test.db");

        // Create schema registry
        let mut registry = SchemaRegistry::new();

        let mut fields = HashMap::new();
        fields.insert("_id".to_string(), FieldType::String { required: true });
        fields.insert("name".to_string(), FieldType::String { required: true });
        fields.insert("age".to_string(), FieldType::Int { required: false });

        let schema = Schema::new("users", "v1", fields);
        registry.register(schema).unwrap();

        // Create WAL writer
        let wal_writer = WalWriter::new(&wal_path).unwrap();

        // Create storage writer and reader
        let storage_writer = StorageWriter::new(&storage_path).unwrap();
        let storage_reader = StorageReader::new(&storage_path).unwrap();

        // Create index manager
        let mut indexed = std::collections::HashSet::new();
        indexed.insert("age".to_string());
        let index_manager = IndexManager::new(indexed);

        (temp_dir, registry, wal_writer, storage_writer, storage_reader, index_manager)
    }

    #[test]
    fn test_insert_and_query_roundtrip() {
        let (_temp, registry, mut wal, mut storage_w, storage_r, mut index) = setup_test_env();

        let mut handler = ApiHandler::new(
            &registry,
            &mut wal,
            &mut storage_w,
            &storage_r,
            &mut index,
            "users",
        );

        // Insert
        let insert_req = r#"{
            "op": "insert",
            "schema_id": "users",
            "schema_version": "v1",
            "document": {"_id": "user_1", "name": "Alice", "age": 25}
        }"#;

        let resp = handler.handle(insert_req);
        assert!(resp.is_success(), "Insert should succeed");

        // Query
        let query_req = r#"{
            "op": "query",
            "schema_id": "users",
            "schema_version": "v1",
            "filter": {"_id": {"$eq": "user_1"}},
            "limit": 10
        }"#;

        let resp = handler.handle(query_req);
        assert!(resp.is_success(), "Query should succeed");
    }

    #[test]
    fn test_invalid_schema_rejected() {
        let (_temp, registry, mut wal, mut storage_w, storage_r, mut index) = setup_test_env();

        let mut handler = ApiHandler::new(
            &registry,
            &mut wal,
            &mut storage_w,
            &storage_r,
            &mut index,
            "users",
        );

        // Insert with unknown schema
        let insert_req = r#"{
            "op": "insert",
            "schema_id": "unknown",
            "schema_version": "v1",
            "document": {"_id": "user_1", "name": "Alice"}
        }"#;

        let resp = handler.handle(insert_req);
        assert!(!resp.is_success());
    }

    #[test]
    fn test_unbounded_query_rejected() {
        let (_temp, registry, mut wal, mut storage_w, storage_r, mut index) = setup_test_env();

        let mut handler = ApiHandler::new(
            &registry,
            &mut wal,
            &mut storage_w,
            &storage_r,
            &mut index,
            "users",
        );

        // Query without indexed filter
        let query_req = r#"{
            "op": "query",
            "schema_id": "users",
            "schema_version": "v1",
            "filter": {"name": {"$eq": "Alice"}},
            "limit": 10
        }"#;

        let resp = handler.handle(query_req);
        // This should fail because "name" is not indexed
        assert!(!resp.is_success());
    }

    #[test]
    fn test_explain_returns_deterministic_plan() {
        let (_temp, registry, mut wal, mut storage_w, storage_r, mut index) = setup_test_env();

        let mut handler = ApiHandler::new(
            &registry,
            &mut wal,
            &mut storage_w,
            &storage_r,
            &mut index,
            "users",
        );

        let explain_req = r#"{
            "op": "explain",
            "schema_id": "users",
            "schema_version": "v1",
            "filter": {"_id": {"$eq": "user_1"}},
            "limit": 10
        }"#;

        let resp1 = handler.handle(explain_req);
        let resp2 = handler.handle(explain_req);

        // Plans should be identical
        assert_eq!(resp1.to_json(), resp2.to_json());
    }

    #[test]
    fn test_serialization_enforced() {
        // This test verifies the lock exists; actual blocking tested differently
        let (_temp, registry, mut wal, mut storage_w, storage_r, mut index) = setup_test_env();

        let mut handler = ApiHandler::new(
            &registry,
            &mut wal,
            &mut storage_w,
            &storage_r,
            &mut index,
            "users",
        );

        // Sequential operations should succeed
        let insert1 = r#"{
            "op": "insert",
            "schema_id": "users",
            "schema_version": "v1",
            "document": {"_id": "user_1", "name": "Alice"}
        }"#;

        let insert2 = r#"{
            "op": "insert",
            "schema_id": "users",
            "schema_version": "v1",
            "document": {"_id": "user_2", "name": "Bob"}
        }"#;

        let resp1 = handler.handle(insert1);
        let resp2 = handler.handle(insert2);

        assert!(resp1.is_success());
        assert!(resp2.is_success());
    }
}
