// src/security/auth.rs - Authentication service
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;
use log::{debug, error};

use crate::config::Config;

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiration timestamp
    pub exp: u64,
    /// User roles
    pub roles: Vec<String>,
}

/// Authentication error
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Failed to encode JWT: {0}")]
    JwtEncodeError(#[from] jsonwebtoken::errors::Error),
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("User not found")]
    UserNotFound,
}

/// Authentication service
pub struct AuthService {
    /// JWT secret
    jwt_secret: String,
    
    /// Refresh token secret
    refresh_secret: String,
    
    /// Token expiry time
    token_expiry: Duration,
    
    /// Refresh token expiry time
    refresh_expiry: Duration,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(config: &Config) -> Self {
        let token_expiry = Duration::from_secs(config.token_expiry_hours * 60 * 60);
        let refresh_expiry = Duration::from_secs(config.token_expiry_hours * 60 * 60 * 24 * 7); // 7 days
        
        Self {
            jwt_secret: config.jwt_secret.clone(),
            refresh_secret: config.refresh_secret.clone(),
            token_expiry,
            refresh_expiry,
        }
    }
    
    /// Generate a JWT token for a user
    pub fn generate_token(&self, user_id: Uuid, roles: Vec<String>) -> Result<String, AuthError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        
        let claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp: now + self.token_expiry.as_secs(),
            roles,
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;
        
        debug!("Generated JWT token for user {}", user_id);
        
        Ok(token)
    }
    
    /// Generate a refresh token for a user
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        
        let claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp: now + self.refresh_expiry.as_secs(),
            roles: vec![],
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.refresh_secret.as_bytes()),
        )?;
        
        debug!("Generated refresh token for user {}", user_id);
        
        Ok(token)
    }
    
    /// Validate a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        ).map_err(|e| {
            error!("JWT validation error: {}", e);
            AuthError::InvalidToken
        })?;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        
        if token_data.claims.exp < now {
            return Err(AuthError::TokenExpired);
        }
        
        Ok(token_data.claims)
    }
    
    /// Validate a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.refresh_secret.as_bytes()),
            &validation,
        ).map_err(|e| {
            error!("Refresh token validation error: {}", e);
            AuthError::InvalidToken
        })?;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        
        if token_data.claims.exp < now {
            return Err(AuthError::TokenExpired);
        }
        
        Ok(token_data.claims)
    }
    
    /// Get user ID from JWT token
    pub fn get_user_id_from_token(&self, token: &str) -> Result<Uuid, AuthError> {
        let claims = self.validate_token(token)?;
        
        Uuid::parse_str(&claims.sub).map_err(|_| {
            error!("Invalid user ID in token: {}", claims.sub);
            AuthError::InvalidToken
        })
    }
    
    /// Get user roles from JWT token
    pub fn get_user_roles_from_token(&self, token: &str) -> Result<Vec<String>, AuthError> {
        let claims = self.validate_token(token)?;
        
        Ok(claims.roles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> Config {
        Config {
            jwt_secret: "test_jwt_secret".to_string(),
            refresh_secret: "test_refresh_secret".to_string(),
            token_expiry_hours: 1,
            ..Config::default()
        }
    }
    
    #[test]
    fn test_generate_and_validate_token() {
        let config = create_test_config();
        let auth_service = AuthService::new(&config);
        
        let user_id = Uuid::new_v4();
        let roles = vec!["user".to_string(), "admin".to_string()];
        
        let token = auth_service.generate_token(user_id, roles.clone()).unwrap();
        
        let claims = auth_service.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.roles, roles);
    }
    
    #[test]
    fn test_generate_and_validate_refresh_token() {
        let config = create_test_config();
        let auth_service = AuthService::new(&config);
        
        let user_id = Uuid::new_v4();
        
        let token = auth_service.generate_refresh_token(user_id).unwrap();
        
        let claims = auth_service.validate_refresh_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert!(claims.roles.is_empty());
    }
    
    #[test]
    fn test_get_user_id_from_token() {
        let config = create_test_config();
        let auth_service = AuthService::new(&config);
        
        let user_id = Uuid::new_v4();
        let roles = vec!["user".to_string()];
        
        let token = auth_service.generate_token(user_id, roles).unwrap();
        
        let result = auth_service.get_user_id_from_token(&token).unwrap();
        
        assert_eq!(result, user_id);
    }
    
    #[test]
    fn test_get_user_roles_from_token() {
        let config = create_test_config();
        let auth_service = AuthService::new(&config);
        
        let user_id = Uuid::new_v4();
        let roles = vec!["user".to_string(), "admin".to_string()];
        
        let token = auth_service.generate_token(user_id, roles.clone()).unwrap();
        
        let result = auth_service.get_user_roles_from_token(&token).unwrap();
        
        assert_eq!(result, roles);
    }
}
