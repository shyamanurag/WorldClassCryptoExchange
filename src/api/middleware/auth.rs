use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, Ready};
use futures::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::config::Config;
use super::super::handlers::user::TokenClaims;

pub struct AuthenticationMiddleware {
    jwt_secret: String,
}

impl AuthenticationMiddleware {
    pub fn new(config: &Config) -> Self {
        Self {
            jwt_secret: config.jwt.secret.clone(),
        }
    }
}

impl<S, B> Transform<S> for AuthenticationMiddleware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddlewareService {
            service,
            jwt_secret: self.jwt_secret.clone(),
        })
    }
}

pub struct AuthenticationMiddlewareService<S> {
    service: S,
    jwt_secret: String,
}

impl<S, B> Service for AuthenticationMiddlewareService<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for public routes
        let path = req.path().to_string();
        if path.contains("/auth/login") || path.contains("/auth/register") {
            return Box::pin(self.service.call(req));
        }

        // Get the Authorization header
        let auth_header = req.headers().get("Authorization");
        if auth_header.is_none() {
            return Box::pin(async move {
                Ok(req.into_response(
                    actix_web::HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Missing Authorization header"
                        }))
                        .into_body(),
                ))
            });
        }

        let auth_value = auth_header.unwrap().to_str().unwrap_or("");
        if !auth_value.starts_with("Bearer ") {
            return Box::pin(async move {
                Ok(req.into_response(
                    actix_web::HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Invalid Authorization header format"
                        }))
                        .into_body(),
                ))
            });
        }

        let token = &auth_value[7..]; // Remove "Bearer " prefix

        // Validate JWT token
        let token_result = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        );

        match token_result {
            Ok(token_data) => {
                // Token is valid, add the user ID to the request extensions
                req.extensions_mut().insert(token_data.claims);
                Box::pin(self.service.call(req))
            }
            Err(_) => Box::pin(async move {
                Ok(req.into_response(
                    actix_web::HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Invalid or expired token"
                        }))
                        .into_body(),
                ))
            }),
        }
    }
}
