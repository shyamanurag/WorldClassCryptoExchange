use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;

mod config;
mod trading_engine;
mod wallet;
mod api;
mod security;
mod db;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    component: Component,
}

#[derive(Subcommand)]
enum Component {
    /// Run the trading engine component
    TradingEngine,
    
    /// Run the wallet system component
    WalletSystem,
    
    /// Run the API gateway component
    ApiGateway,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    utils::logging::init_logger();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Load configuration
    let config = config::load_config()?;
    
    // Initialize metrics
    utils::metrics::init_metrics(&config)?;
    
    // Run the selected component
    match cli.component {
        Component::TradingEngine => {
            info!("Starting trading engine...");
            trading_engine::run(config).await?;
        },
        Component::WalletSystem => {
            info!("Starting wallet system...");
            wallet::run(config).await?;
        },
        Component::ApiGateway => {
            info!("Starting API gateway...");
            api::run(config).await?;
        },
    }
    
    Ok(())
}
