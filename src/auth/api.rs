//! # Auth API Endpoints
//!
//! HTTP API endpoints for authentication.

use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::crypto::PasswordPolicy;
use super::errors::{AuthError, AuthResult};
use super::jwt::{JwtConfig, JwtManager, TokenResponse};
use super::rls::RlsContext;
use super::session::{SessionConfig, SessionManager, SessionRepository};
use super::user::{LoginRequest, SignupRequest, User, UserRepository};

/// Auth service combining all auth components
pub struct AuthService<U: UserRepository, S: SessionRepository> {
    user_repo: Arc<U>,
    session_manager: SessionManager<S>,
    jwt_manager: JwtManager,
    password_policy: PasswordPolicy,
}

impl<U: UserRepository, S: SessionRepository> AuthService<U, S> {
    pub fn new(
        user_repo: U,
        session_repo: S,
        jwt_config: JwtConfig,
        session_config: SessionConfig,
        password_policy: PasswordPolicy,
    ) -> Self {
        Self {
            user_repo: Arc::new(user_repo),
            session_manager: SessionManager::new(session_config, session_repo),
            jwt_manager: JwtManager::new(jwt_config),
            password_policy,
        }
    }
    
    /// Register a new user
    pub fn signup(&self, request: SignupRequest) -> AuthResult<(User, TokenResponse)> {
        // Check if email already exists
        if self.user_repo.email_exists(&request.email)? {
            return Err(AuthError::EmailAlreadyExists);
        }
        
        // Create user
        let mut user = User::new(request.email, &request.password, &self.password_policy)?;
        if let Some(metadata) = request.metadata {
            user.metadata = Some(metadata);
        }
        
        // Store user
        self.user_repo.create(&user)?;
        
        // Create session
        let (_, refresh_token) = self.session_manager.create_session(
            user.id,
            None,
            None,
        )?;
        
        // Generate tokens
        let access_token = self.jwt_manager.generate_access_token(&user)?;
        let token_response = TokenResponse::new(
            access_token,
            refresh_token,
            self.jwt_manager.get_expiration(),
        );
        
        Ok((user, token_response))
    }
    
    /// Authenticate a user
    pub fn login(&self, request: LoginRequest) -> AuthResult<(User, TokenResponse)> {
        // Find user by email
        let user = self.user_repo.find_by_email(&request.email)?
            .ok_or(AuthError::InvalidCredentials)?;
        
        // Verify password
        if !user.verify_password(&request.password)? {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Create session
        let (_, refresh_token) = self.session_manager.create_session(
            user.id,
            None,
            None,
        )?;
        
        // Generate tokens
        let access_token = self.jwt_manager.generate_access_token(&user)?;
        let token_response = TokenResponse::new(
            access_token,
            refresh_token,
            self.jwt_manager.get_expiration(),
        );
        
        Ok((user, token_response))
    }
    
    /// Refresh access token
    pub fn refresh(&self, refresh_token: &str) -> AuthResult<TokenResponse> {
        // Refresh session (invalidates old token)
        let (_, new_refresh_token) = self.session_manager.refresh_session(refresh_token)?;
        
        // Get session to find user
        let session = self.session_manager.validate_refresh_token(&new_refresh_token)?;
        
        // Get user
        let user = self.user_repo.find_by_id(session.user_id)?
            .ok_or(AuthError::InvalidCredentials)?;
        
        // Generate new access token
        let access_token = self.jwt_manager.generate_access_token(&user)?;
        
        Ok(TokenResponse::new(
            access_token,
            new_refresh_token,
            self.jwt_manager.get_expiration(),
        ))
    }
    
    /// Logout (invalidate session)
    pub fn logout(&self, refresh_token: &str) -> AuthResult<()> {
        let session = self.session_manager.validate_refresh_token(refresh_token)?;
        self.session_manager.revoke_session(session.id)
    }
    
    /// Get user by ID
    pub fn get_user(&self, user_id: Uuid) -> AuthResult<User> {
        self.user_repo.find_by_id(user_id)?
            .ok_or(AuthError::InvalidCredentials)
    }
    
    /// Validate an access token and return RLS context
    pub fn validate_access_token(&self, token: &str) -> AuthResult<RlsContext> {
        let claims = self.jwt_manager.validate_token(token)?;
        let user_id = JwtManager::get_user_id(&claims)?;
        Ok(RlsContext::authenticated(user_id))
    }
}

// ==================
// HTTP Request/Response Types
// ==================

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub user: UserResponse,
    #[serde(flatten)]
    pub tokens: TokenResponse,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    #[serde(flatten)]
    pub tokens: TokenResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            created_at: user.created_at,
            metadata: user.metadata,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

impl From<AuthError> for ErrorResponse {
    fn from(err: AuthError) -> Self {
        Self {
            error: err.to_string(),
            code: err.status_code(),
        }
    }
}

// ==================
// HTTP Error Conversion
// ==================

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let code = self.status_code();
        let body = Json(ErrorResponse::from(self));
        
        let status = StatusCode::from_u16(code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::session::InMemorySessionRepository;
    use crate::auth::user::InMemoryUserRepository;
    
    fn create_test_service() -> AuthService<InMemoryUserRepository, InMemorySessionRepository> {
        AuthService::new(
            InMemoryUserRepository::new(),
            InMemorySessionRepository::new(),
            JwtConfig::default(),
            SessionConfig::default(),
            PasswordPolicy::default(),
        )
    }
    
    #[test]
    fn test_signup() {
        let service = create_test_service();
        
        let request = SignupRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            metadata: None,
        };
        
        let (user, tokens) = service.signup(request).unwrap();
        
        assert_eq!(user.email, "test@example.com");
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
    }
    
    #[test]
    fn test_signup_duplicate_email() {
        let service = create_test_service();
        
        let request = SignupRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            metadata: None,
        };
        
        service.signup(request.clone()).unwrap();
        let result = service.signup(request);
        
        assert!(matches!(result, Err(AuthError::EmailAlreadyExists)));
    }
    
    #[test]
    fn test_login() {
        let service = create_test_service();
        
        // Signup first
        let signup = SignupRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            metadata: None,
        };
        service.signup(signup).unwrap();
        
        // Login
        let login = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        let (user, tokens) = service.login(login).unwrap();
        
        assert_eq!(user.email, "test@example.com");
        assert!(!tokens.access_token.is_empty());
    }
    
    #[test]
    fn test_login_wrong_password() {
        let service = create_test_service();
        
        // Signup first
        let signup = SignupRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            metadata: None,
        };
        service.signup(signup).unwrap();
        
        // Login with wrong password
        let login = LoginRequest {
            email: "test@example.com".to_string(),
            password: "wrong_password".to_string(),
        };
        let result = service.login(login);
        
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }
    
    #[test]
    fn test_refresh_token_flow() {
        let service = create_test_service();
        
        // Signup
        let signup = SignupRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            metadata: None,
        };
        let (_, tokens) = service.signup(signup).unwrap();
        
        // Refresh
        let new_tokens = service.refresh(&tokens.refresh_token).unwrap();
        
        assert!(!new_tokens.access_token.is_empty());
        assert_ne!(new_tokens.refresh_token, tokens.refresh_token);
    }
    
    #[test]
    fn test_logout() {
        let service = create_test_service();
        
        // Signup
        let signup = SignupRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            metadata: None,
        };
        let (_, tokens) = service.signup(signup).unwrap();
        
        // Logout
        service.logout(&tokens.refresh_token).unwrap();
        
        // Refresh should fail
        let result = service.refresh(&tokens.refresh_token);
        assert!(matches!(result, Err(AuthError::SessionRevoked)));
    }
    
    #[test]
    fn test_access_token_validation() {
        let service = create_test_service();
        
        // Signup
        let signup = SignupRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            metadata: None,
        };
        let (user, tokens) = service.signup(signup).unwrap();
        
        // Validate access token
        let ctx = service.validate_access_token(&tokens.access_token).unwrap();
        
        assert!(ctx.is_authenticated);
        assert_eq!(ctx.user_id, Some(user.id));
    }
}
