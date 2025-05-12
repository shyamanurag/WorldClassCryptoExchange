use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};

use crate::models::User;
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub two_factor_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
}

// Create a new user
pub async fn create_user(
    req: web::Json<RegisterRequest>,
    config: web::Data<Config>,
) -> impl Responder {
    // Implementation would include:
    // 1. Validate the input
    // 2. Check if user already exists
    // 3. Hash the password
    // 4. Create the user in the database
    // 5. Return the created user

    // Simplified mock implementation
    let user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email.clone(),
        first_name: req.first_name.clone(),
        last_name: req.last_name.clone(),
        created_at: Utc::now().timestamp() as u64,
        two_factor_enabled: false,
        role: "user".to_string(),
        kyc_status: "none".to_string(),
        last_login: None,
    };

    // Create JWT token
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = TokenClaims {
        sub: user.id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
        role: user.role.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    ).expect("JWT token creation");

    HttpResponse::Created().json(LoginResponse {
        user,
        token,
    })
}

// Get all users (admin only)
pub async fn get_users() -> impl Responder {
    // Implementation with proper auth checks and pagination
    HttpResponse::Ok().json(vec![])
}

// Get a specific user
pub async fn get_user(path: web::Path<Uuid>) -> impl Responder {
    let user_id = path.into_inner();
    // Implementation would fetch from database
    HttpResponse::Ok().json(User {
        id: user_id,
        username: "user123".to_string(),
        email: "user@example.com".to_string(),
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        created_at: Utc::now().timestamp() as u64,
        two_factor_enabled: false,
        role: "user".to_string(),
        kyc_status: "none".to_string(),
        last_login: None,
    })
}

// Update a user
pub async fn update_user(
    path: web::Path<Uuid>,
    _req: web::Json<serde_json::Value>,
) -> impl Responder {
    let user_id = path.into_inner();
    // Implementation would update in database
    HttpResponse::Ok().json(User {
        id: user_id,
        username: "updated_user".to_string(),
        email: "updated@example.com".to_string(),
        first_name: Some("Updated".to_string()),
        last_name: Some("User".to_string()),
        created_at: Utc::now().timestamp() as u64,
        two_factor_enabled: false,
        role: "user".to_string(),
        kyc_status: "none".to_string(),
        last_login: None,
    })
}

// Delete a user
pub async fn delete_user(path: web::Path<Uuid>) -> impl Responder {
    let _user_id = path.into_inner();
    // Implementation would delete from database
    HttpResponse::NoContent().finish()
}

// User login
pub async fn login(
    req: web::Json<LoginRequest>,
    config: web::Data<Config>,
) -> impl Responder {
    // Implementation would include:
    // 1. Validate credentials
    // 2. Check 2FA if enabled
    // 3. Create and return JWT token

    // Simplified mock implementation
    let user = User {
        id: Uuid::new_v4(),
        username: "logged_in_user".to_string(),
        email: req.email.clone(),
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        created_at: Utc::now().timestamp() as u64,
        two_factor_enabled: false,
        role: "user".to_string(),
        kyc_status: "none".to_string(),
        last_login: Some(Utc::now().timestamp() as u64),
    };

    // Create JWT token
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = TokenClaims {
        sub: user.id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
        role: user.role.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    ).expect("JWT token creation");

    HttpResponse::Ok().json(LoginResponse {
        user,
        token,
    })
}

// User logout
pub async fn logout() -> impl Responder {
    // Implementation would invalidate token if using a token blacklist
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully logged out"
    }))
}

// Refresh JWT token
pub async fn refresh_token(
    config: web::Data<Config>,
) -> impl Responder {
    // Implementation would verify the refresh token and issue a new access token
    
    // Simplified mock implementation
    let user_id = Uuid::new_v4();
    
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
        role: "user".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    ).expect("JWT token creation");

    HttpResponse::Ok().json(serde_json::json!({
        "token": token
    }))
}

// Get current authenticated user
pub async fn get_current_user() -> impl Responder {
    // Implementation would extract user from JWT token
    
    // Simplified mock implementation
    let user = User {
        id: Uuid::new_v4(),
        username: "current_user".to_string(),
        email: "current@example.com".to_string(),
        first_name: Some("Current".to_string()),
        last_name: Some("User".to_string()),
        created_at: Utc::now().timestamp() as u64,
        two_factor_enabled: false,
        role: "user".to_string(),
        kyc_status: "none".to_string(),
        last_login: Some(Utc::now().timestamp() as u64),
    };

    HttpResponse::Ok().json(user)
}

// Enable 2FA
pub async fn enable_2fa() -> impl Responder {
    // Implementation would generate a 2FA secret and QR code
    HttpResponse::Ok().json(serde_json::json!({
        "secret": "JBSWY3DPEHPK3PXP",
        "qrCode": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAQAAAAEAAQMAAABmvDolAAAABlBMVEX///8AAABVwtN+AAABA0lEQVR42uyYwQ3CQAwEXQEVBJdAA+bcAA1AA1RABdAARdAAkTgk5EeRNyxZyj4zZe2V7exu0k0URVEUvUHEVK90qee6P5Pc6S29e8pCL3pJtR6UiZ7Kj9YunT1S7ZLo+W+tO7vKWmtX/saaLXjAIYcccsghhxxyyCGHHHLIIYcccsghhxxy+FOH4yT51UUPjsfn1CmKoijKLqLnsVxz3ZhZJmPcWB4iZo3EUJ7IxLKa6+pydji5VeWZHiGbnGT5NR+sFqxs5Y0V15hYIRlZeXlQVlZXX+tB3Vh1jZU31ldjZYwnHPBu3iuP5sTLH3Aj3CQ3G7fW7XfT3KD3w336AbKiKIqi/9QFUKmZrMy4XdoAAAAASUVORK5CYII="
    }))
}

// Verify 2FA
pub async fn verify_2fa(
    req: web::Json<serde_json::Value>,
) -> impl Responder {
    // Implementation would verify the 2FA token
    let verified = req.get("token").is_some();
    
    HttpResponse::Ok().json(serde_json::json!({
        "verified": verified
    }))
}

// Disable 2FA
pub async fn disable_2fa(
    req: web::Json<serde_json::Value>,
) -> impl Responder {
    // Implementation would disable 2FA after verifying token
    let verified = req.get("token").is_some();
    
    HttpResponse::Ok().json(serde_json::json!({
        "disabled": verified
    }))
}
