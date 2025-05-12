

use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

use crate::defi::{
    DeFiManager, ProtocolRegistry, ProtocolAdapter, UniswapAdapter, AaveLendingAdapter,
    Protocol, Pool, Position, DeFiTransaction, AssetAmount, PositionId, UserId,
};

// API request types
#[derive(Debug, Deserialize)]
pub struct CreatePositionRequest {
    pub protocol_id: String,
    pub pool_id: String,
    pub assets: Vec<AssetAmount>,
}

#[derive(Debug, Deserialize)]
pub struct ClosePositionRequest {
    pub position_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct HarvestRewardsRequest {
    pub position_id: Uuid,
}

// API response types
#[derive(Debug, Serialize)]
pub struct CreatePositionResponse {
    pub position: Position,
    pub transaction: DeFiTransaction,
}

#[derive(Debug, Serialize)]
pub struct ClosePositionResponse {
    pub position: Position,
    pub transaction: DeFiTransaction,
}

#[derive(Debug, Serialize)]
pub struct HarvestRewardsResponse {
    pub position: Position,
    pub transaction: DeFiTransaction,
}

#[derive(Debug, Serialize)]
pub struct ListProtocolsResponse {
    pub protocols: Vec<Protocol>,
}

#[derive(Debug, Serialize)]
pub struct ListPoolsResponse {
    pub pools: Vec<Pool>,
}

#[derive(Debug, Serialize)]
pub struct ListPositionsResponse {
    pub positions: Vec<Position>,
}

#[derive(Debug, Serialize)]
pub struct PortfolioValueResponse {
    pub value: Decimal,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct PositionHealthResponse {
    pub position_id: PositionId,
    pub health_factor: Decimal,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct PositionsHealthResponse {
    pub positions: Vec<PositionHealthResponse>,
}

// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// AppState for DeFi service
pub struct DeFiAppState {
    pub defi_manager: Arc<DeFiManager>,
}

// Create the DeFi service
pub async fn create_defi_service() -> Result<(Router, Arc<DeFiManager>)> {
    // Create registry and adapters
    let registry = Arc::new(ProtocolRegistry::new());
    
    // Initialize Uniswap adapter
    let uniswap_adapter = Arc::new(UniswapAdapter::new());
    uniswap_adapter.initialize_pools().await?;
    registry.register_adapter(uniswap_adapter).await?;
    
    // Initialize Aave adapter
    let aave_adapter = Arc::new(AaveLendingAdapter::new());
    aave_adapter.initialize_pools().await?;
    registry.register_adapter(aave_adapter).await?;
    
    // Create DeFi manager
    let defi_manager = Arc::new(DeFiManager::new(registry));
    
    // Create app state
    let app_state = Arc::new(DeFiAppState {
        defi_manager: defi_manager.clone(),
    });
    
    // Create router
    let router = Router::new()
        // Protocols
        .route("/protocols", get(list_protocols))
        .route("/protocols/:protocol_id/pools", get(list_pools))
        
        // Positions
        .route("/positions", get(list_positions))
        .route("/positions", post(create_position))
        .route("/positions/:position_id", get(get_position))
        .route("/positions/:position_id/close", post(close_position))
        .route("/positions/:position_id/harvest", post(harvest_rewards))
        
        // Portfolio
        .route("/portfolio/value", get(get_portfolio_value))
        .route("/portfolio/health", get(check_positions_health))
        
        .with_state(app_state);
    
    Ok((router, defi_manager))
}

// Handler functions

// List all available protocols
async fn list_protocols(
    State(state): State<Arc<DeFiAppState>>,
) -> impl IntoResponse {
    match state.defi_manager.list_protocols().await {
        Ok(protocols) => {
            let response = ListProtocolsResponse { protocols };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// List pools for a specific protocol
async fn list_pools(
    State(state): State<Arc<DeFiAppState>>,
    Path(protocol_id): Path<String>,
) -> impl IntoResponse {
    match state.defi_manager.list_pools(&protocol_id).await {
        Ok(pools) => {
            let response = ListPoolsResponse { pools };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// List user positions
async fn list_positions(
    State(state): State<Arc<DeFiAppState>>,
    Query(params): Query<UserIdParam>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse { error: "Invalid user ID".to_string() };
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };
    
    match state.defi_manager.get_user_positions(&user_id).await {
        Ok(positions) => {
            let response = ListPositionsResponse { positions };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// Create a new position
async fn create_position(
    State(state): State<Arc<DeFiAppState>>,
    Query(params): Query<UserIdParam>,
    Json(request): Json<CreatePositionRequest>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse { error: "Invalid user ID".to_string() };
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };
    
    match state.defi_manager.create_position(
        user_id,
        &request.protocol_id,
        &request.pool_id,
        request.assets,
    ).await {
        Ok((position, transaction)) => {
            let response = CreatePositionResponse { position, transaction };
            (StatusCode::CREATED, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// Get a specific position
async fn get_position(
    State(state): State<Arc<DeFiAppState>>,
    Path(position_id): Path<Uuid>,
    Query(params): Query<UserIdParam>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse { error: "Invalid user ID".to_string() };
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };
    
    match state.defi_manager.get_user_positions(&user_id).await {
        Ok(positions) => {
            if let Some(position) = positions.iter().find(|p| p.id == position_id) {
                (StatusCode::OK, Json(position))
            } else {
                let error = ErrorResponse { error: "Position not found".to_string() };
                (StatusCode::NOT_FOUND, Json(error))
            }
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// Close a position
async fn close_position(
    State(state): State<Arc<DeFiAppState>>,
    Path(position_id): Path<Uuid>,
    Query(params): Query<UserIdParam>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse { error: "Invalid user ID".to_string() };
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };
    
    match state.defi_manager.close_position(user_id, position_id).await {
        Ok((position, transaction)) => {
            let response = ClosePositionResponse { position, transaction };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// Harvest rewards from a position
async fn harvest_rewards(
    State(state): State<Arc<DeFiAppState>>,
    Path(position_id): Path<Uuid>,
    Query(params): Query<UserIdParam>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse { error: "Invalid user ID".to_string() };
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };
    
    match state.defi_manager.harvest_rewards(user_id, position_id).await {
        Ok((position, transaction)) => {
            let response = HarvestRewardsResponse { position, transaction };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// Get portfolio value
async fn get_portfolio_value(
    State(state): State<Arc<DeFiAppState>>,
    Query(params): Query<UserIdParam>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse { error: "Invalid user ID".to_string() };
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };
    
    match state.defi_manager.get_portfolio_value(&user_id).await {
        Ok(value) => {
            let response = PortfolioValueResponse { 
                value,
                currency: "USD".to_string(), 
            };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// Check health of all positions
async fn check_positions_health(
    State(state): State<Arc<DeFiAppState>>,
    Query(params): Query<UserIdParam>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse { error: "Invalid user ID".to_string() };
            return (StatusCode::BAD_REQUEST, Json(error));
        }
    };
    
    match state.defi_manager.check_positions_health(&user_id).await {
        Ok(health_data) => {
            let positions = health_data.into_iter().map(|(position, health_factor)| {
                let status = if health_factor < Decimal::new(12, 1) {
                    "At Risk".to_string()
                } else if health_factor < Decimal::new(15, 1) {
                    "Warning".to_string()
                } else {
                    "Healthy".to_string()
                };
                
                PositionHealthResponse {
                    position_id: position.id,
                    health_factor,
                    status,
                }
            }).collect();
            
            let response = PositionsHealthResponse { positions };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

// Helper structs

#[derive(Debug, Deserialize)]
pub struct UserIdParam {
    pub user_id: String,
}
