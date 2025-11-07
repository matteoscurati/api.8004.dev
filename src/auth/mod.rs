use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (username or user_id)
    pub exp: usize,       // Expiration time (Unix timestamp)
    pub iat: usize,       // Issued at (Unix timestamp)
}

/// JWT authentication error
#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    MissingToken,
    TokenExpired,
    WrongCredentials,
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::MissingToken => write!(f, "Missing authentication token"),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::WrongCredentials => write!(f, "Wrong credentials"),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authentication token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token has expired"),
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// JWT configuration
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub token_expiration_hours: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set in environment"),
            token_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a valid number"),
        }
    }

    /// Create a new JWT token
    pub fn create_token(&self, username: &str) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let exp = (now + chrono::Duration::hours(self.token_expiration_hours)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = Claims {
            sub: username.to_owned(),
            exp,
            iat,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|_| AuthError::InvalidToken)
    }

    /// Validate a JWT token and return claims
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            }
        })?;

        Ok(token_data.claims)
    }
}

/// Extractor for authenticated requests
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to extract token from Authorization header first
        let token = if let Ok(TypedHeader(Authorization(bearer))) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
        {
            bearer.token().to_string()
        } else {
            // Fallback: try to extract token from query parameter (for WebSocket)
            parts
                .uri
                .query()
                .and_then(|q| {
                    q.split('&')
                        .find_map(|pair| {
                            let mut split = pair.split('=');
                            if split.next() == Some("token") {
                                split.next().map(|t| t.to_string())
                            } else {
                                None
                            }
                        })
                })
                .ok_or(AuthError::MissingToken)?
        };

        // Get JWT config from extensions
        let jwt_config = parts
            .extensions
            .get::<JwtConfig>()
            .ok_or(AuthError::InvalidToken)?;

        // Validate the token
        jwt_config.validate_token(&token)
    }
}

/// Login credentials
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: String,
}

/// Validate user credentials with bcrypt password hashing
pub fn validate_credentials(username: &str, password: &str) -> bool {
    // Get credentials from environment
    let valid_username = std::env::var("AUTH_USERNAME").unwrap_or_else(|_| "admin".to_string());

    // Check if password hash is available
    if let Ok(password_hash) = std::env::var("AUTH_PASSWORD_HASH") {
        // Verify username matches
        if username != valid_username {
            return false;
        }

        // Verify password hash
        match bcrypt::verify(password, &password_hash) {
            Ok(valid) => valid,
            Err(e) => {
                tracing::error!("Password verification error: {}", e);
                false
            }
        }
    } else {
        // Fallback: if no hash provided, check plain password (NOT RECOMMENDED FOR PRODUCTION)
        let plain_password = std::env::var("AUTH_PASSWORD").unwrap_or_else(|_| "changeme".to_string());
        tracing::warn!("Using plain text password! Set AUTH_PASSWORD_HASH for production");
        username == valid_username && password == plain_password
    }
}

/// Utility function for generating password hashes (used by tests and external scripts)
#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use serial_test::serial;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("Failed to hash password");

        // Verify the hash is valid
        assert!(bcrypt::verify(password, &hash).unwrap());

        // Verify wrong password fails
        assert!(!bcrypt::verify("wrong_password", &hash).unwrap());
    }

    #[test]
    #[serial]
    fn test_jwt_token_creation_and_validation() {
        env::set_var("JWT_SECRET", "test-secret-key-that-is-at-least-32-characters-long");
        env::set_var("JWT_EXPIRATION_HOURS", "24");

        let config = JwtConfig::from_env();
        let username = "testuser";

        // Create token
        let token = config.create_token(username).expect("Failed to create token");

        // Validate token
        let claims = config.validate_token(&token).expect("Failed to validate token");
        assert_eq!(claims.sub, username);
    }

    #[test]
    #[serial]
    fn test_jwt_token_invalid() {
        env::set_var("JWT_SECRET", "test-secret-key-that-is-at-least-32-characters-long");
        env::set_var("JWT_EXPIRATION_HOURS", "24");

        let config = JwtConfig::from_env();

        // Invalid token should fail
        let result = config.validate_token("invalid.token.string");
        assert!(result.is_err());

        // Token with wrong secret should fail
        env::set_var("JWT_SECRET", "different-secret-key-that-is-32-chars");
        let config2 = JwtConfig::from_env();

        env::set_var("JWT_SECRET", "test-secret-key-that-is-at-least-32-characters-long");
        let config1 = JwtConfig::from_env();
        let token = config1.create_token("testuser").unwrap();

        let result = config2.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_validate_credentials_with_plain_password() {
        env::set_var("AUTH_USERNAME", "admin");
        env::set_var("AUTH_PASSWORD", "testpass123");
        env::remove_var("AUTH_PASSWORD_HASH");

        // Valid credentials
        assert!(validate_credentials("admin", "testpass123"));

        // Invalid username
        assert!(!validate_credentials("wronguser", "testpass123"));

        // Invalid password
        assert!(!validate_credentials("admin", "wrongpass"));
    }

    #[test]
    #[serial]
    fn test_validate_credentials_with_bcrypt() {
        env::set_var("AUTH_USERNAME", "admin");
        let password = "securepassword";
        let hash = hash_password(password).unwrap();
        env::set_var("AUTH_PASSWORD_HASH", &hash);

        // Valid credentials
        assert!(validate_credentials("admin", password));

        // Invalid password
        assert!(!validate_credentials("admin", "wrongpassword"));

        // Invalid username
        assert!(!validate_credentials("wronguser", password));
    }

    #[test]
    #[serial]
    fn test_jwt_config_loads_from_env() {
        // Just test that config loads without errors
        env::set_var("JWT_SECRET", "test-secret-key-that-is-at-least-32-characters-long");
        env::set_var("JWT_EXPIRATION_HOURS", "48");

        let config = JwtConfig::from_env();

        // Verify config has values (any non-empty values)
        assert!(!config.secret.is_empty());
        assert!(config.token_expiration_hours > 0);
    }
}
