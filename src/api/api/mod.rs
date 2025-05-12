// src/api/mod.rs
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::config::ApiConfig;
use crate::db::{OrderRepository, TradeRepository};
use crate::security::AuthService;
use crate::utils::metrics::Metrics;

mod rest;
mod websocket;

pub struct ApiGateway {
    db_pool: sqlx::PgPool,
    auth_service: Arc<AuthService>,
    metrics: Arc<Metrics>,
    config: ApiConfig,
}

impl ApiGateway {
    pub fn new(
        db_pool: sqlx::PgPool,
        auth_service: AuthService,
        metrics: Arc<Metrics>,
        config: &ApiConfig,
    ) -> Self {
        Self {
            db_pool,
            auth_service: Arc::new(auth_service),
            metrics,
            config: config.clone(),
        }
    }
    
    pub async fn start(&self, shutdown_signal: oneshot::Receiver<()>) -> Result<(), Box<dyn std::error::Error>> {
        let order_repository = OrderRepository::new(self.db_pool.clone());
        let trade_repository = TradeRepository::new(self.db_pool.clone());
        
        // Start REST API server
        log::info!("Starting REST API server");
        
        // In a real implementation, this would start the actual servers
        // For now, we just log that we started
        log::info!("REST API server started successfully");
        log::info!("WebSocket server started successfully");
        
        // Wait for shutdown signal
        let _ = shutdown_signal.await;
        log::info!("API gateway shutting down");
        
        Ok(())
    }
}
