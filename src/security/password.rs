// src/security/password.rs - Password hashing and verification
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;
use log::error;

/// Password service error
#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Failed to hash password: {0}")]
    HashingError(String),
    
    #[error("Failed to verify password: {0}")]
    VerificationError(String),
}

/// Password service for hashing and verifying passwords
pub struct PasswordService;

impl PasswordService {
    /// Hash a password
    pub fn hash_password(password: &str) -> Result<String, PasswordError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| {
                error!("Password hashing error: {}", e);
                PasswordError::HashingError(e.to_string())
            })
    }
    
    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            error!("Password hash parsing error: {}", e);
            PasswordError::VerificationError(e.to_string())
        })?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_and_verify_password() {
        let password = "test_password";
        
        let hash = PasswordService::hash_password(password).unwrap();
        
        let result = PasswordService::verify_password(password, &hash).unwrap();
        
        assert!(result);
    }
    
    #[test]
    fn test_verify_wrong_password() {
        let password = "test_password";
        let wrong_password = "wrong_password";
        
        let hash = PasswordService::hash_password(password).unwrap();
        
        let result = PasswordService::verify_password(wrong_password, &hash).unwrap();
        
        assert!(!result);
    }
}
