// src/main.rs - Main application entry point
use anyhow::{anyhow, Context, Result};
use clap::{App, Arg, SubCommand};
use log::{info, error};
use std::sync::Arc;
use tokio::signal;

// Import local modules
mod config;
mod api;
mod db;
mod security;
mod trading_engine;
mod wallet;
mod core;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = App::new("WorldClassCryptoExchange")
        .version("0.1.0")
        .author("Shyam Anurag")
        .about("High-performance cryptocurrency trading platform")
        .subcommand(
            SubCommand::with_name("trading-engine")
                .about("Start the trading engine")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .help("Sets the API port")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("api-gateway")
                .about("Start the API gateway only")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .help("Sets the API port")
                        .takes_value(true),
                ),
        )
        .get_matches();

    // Initialize logging
    utils::logging::init_logger();
    
    // Load configuration
    let mut config = config::Config::load()?;
    
    // Override port from command line if provided
    if let Some(port_str) = matches
        .subcommand_matches("trading-engine")
        .and_then(|m| m.value_of("port"))
        .or_else(|| matches.subcommand_matches("api-gateway")
                            .and_then(|m| m.value_of("port")))
    {
        if let Ok(port) = port_str.parse::<u16>() {
            config.api_port = port;
        }
    }
    
    // Initialize components based on subcommand
    match matches.subcommand_name() {
        Some("trading-engine") => {
            info!("Starting in trading engine mode");
            
            // Initialize metrics
            let metrics = utils::metrics::init_metrics(&config)?;
            
            // Initialize repositories
            let db_pool = db::init_database(&config.database_url).await?;
            let user_repo = Arc::new(db::UserRepository::new(db_pool.clone()));
            let asset_repo = Arc::new(db::AssetRepository::new(db_pool.clone()));
            let trading_pair_repo = Arc::new(db::TradingPairRepository::new(db_pool.clone()));
            let order_repo = Arc::new(db::OrderRepository::new(db_pool.clone()));
            let trade_repo = Arc::new(db::TradeRepository::new(db_pool.clone()));
            let wallet_repo = Arc::new(db::WalletRepository::new(db_pool.clone()));
            let account_repo = Arc::new(db::AccountRepository::new(db_pool.clone()));
            let deposit_repo = Arc::new(db::DepositRepository::new(db_pool.clone()));
            let withdrawal_repo = Arc::new(db::WithdrawalRepository::new(db_pool.clone()));
            
            // Initialize security
            let auth_service = Arc::new(security::AuthService::new(&config));
            
            // Initialize trading engine
            let manager = Arc::new(trading_engine::EngineManager::new(
                metrics.clone(),
                order_repo.clone(),
                trade_repo.clone(),
            ));
            
            // Add trading pairs
            for pair in config.trading_pairs.clone() {
                info!("Adding trading pair: {}", pair);
                manager.add_symbol(pair).await?;
            }
            
            // Initialize wallet system
            let wallet_handle = tokio::spawn(async move {
                if let Err(e) = wallet::run(config.clone(), false).await {
                    error!("Wallet system error: {}", e);
                }
            });
            
            // Initialize API service
            let mut api_service = api::ApiService::new(
                Arc::clone(&manager),
                user_repo,
                asset_repo,
                trading_pair_repo,
                order_repo,
                trade_repo,
                wallet_repo,
                account_repo,
                deposit_repo,
                withdrawal_repo,
                Arc::clone(&auth_service),
            );
            
            // Start API service
            api_service.start("0.0.0.0", config.api_port).await?;
            
            info!("All components started successfully");
            
            // Wait for shutdown signal
            wait_for_shutdown().await?;
            
            // Shutdown API service
            api_service.stop().await?;
            
            // Wait for wallet system to shut down
            if let Err(e) = wallet_handle.await {
                error!("Error waiting for wallet system to shut down: {}", e);
            }
        },
        Some("api-gateway") => {
            info!("Starting in API gateway mode");
            
            // Initialize metrics
            let metrics = utils::metrics::init_metrics(&config)?;
            
            // Initialize repositories
            let db_pool = db::init_database(&config.database_url).await?;
            let user_repo = Arc::new(db::UserRepository::new(db_pool.clone()));
            let asset_repo = Arc::new(db::AssetRepository::new(db_pool.clone()));
            let trading_pair_repo = Arc::new(db::TradingPairRepository::new(db_pool.clone()));
            let order_repo = Arc::new(db::OrderRepository::new(db_pool.clone()));
            let trade_repo = Arc::new(db::TradeRepository::new(db_pool.clone()));
            let wallet_repo = Arc::new(db::WalletRepository::new(db_pool.clone()));
            let account_repo = Arc::new(db::AccountRepository::new(db_pool.clone()));
            let deposit_repo = Arc::new(db::DepositRepository::new(db_pool.clone()));
            let withdrawal_repo = Arc::new(db::WithdrawalRepository::new(db_pool.clone()));
            
            // Initialize security
            let auth_service = Arc::new(security::AuthService::new(&config));
            
            // Initialize API service without trading engine
            let mut api_service = api::ApiService::new_without_engine(
                user_repo,
                asset_repo,
                trading_pair_repo,
                order_repo,
                trade_repo,
                wallet_repo,
                account_repo,
                deposit_repo,
                withdrawal_repo,
                Arc::clone(&auth_service),
            );
            
            // Start API service
            api_service.start("0.0.0.0", config.api_port).await?;
            
            info!("API gateway started successfully");
            
            // Wait for shutdown signal
            wait_for_shutdown().await?;
            
            // Shutdown API service
            api_service.stop().await?;
        },
        _ => {
            return Err(anyhow!("Please specify a valid subcommand. Use --help for more information."));
        }
    }
    
    info!("WorldClass Crypto Exchange shut down gracefully");
    
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C)
async fn wait_for_shutdown() -> Result<()> {
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, starting graceful shutdown");
            Ok(())
        }
        Err(e) => {
            error!("Failed to listen for shutdown signal: {}", e);
            Err(anyhow!("Failed to listen for shutdown signal"))
        }
    }
}
