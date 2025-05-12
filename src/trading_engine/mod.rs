
rustuse std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{self, Duration};
use anyhow::Result;

use crate::models::{Order, Trade};
use crate::utils::metrics::MetricsCollector;
use crate::config::Config;

pub mod order_book;
pub mod matching_engine;
pub mod risk_management;
pub mod market_data;

use matching_engine::MatchingEngine;
use risk_management::RiskManager;

/// The TradingEngine is the main entry point for order processing.
/// It coordinates the matching engine, risk manager, and other components.
pub struct TradingEngine {
    /// The matching engine
    matching_engine: Arc<RwLock<MatchingEngine>>,
    
    /// The risk manager
    risk_manager: Arc<RiskManager>,
    
    /// Order channel for receiving orders
    order_rx: mpsc::Receiver<Order>,
    
    /// Order channel for sending orders
    order_tx: mpsc::Sender<Order>,
    
    /// Trade channel for receiving trades
    trade_rx: mpsc::Receiver<Trade>,
    
    /// Trade channel for sending trades
    trade_tx: mpsc::Sender<Trade>,
    
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
}

impl TradingEngine {
    /// Creates a new trading engine
    pub fn new(
        matching_engine: Arc<RwLock<MatchingEngine>>,
        risk_manager: Arc<RiskManager>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let (order_tx, order_rx) = mpsc::channel(1000);
        let (trade_tx, trade_rx) = mpsc::channel(1000);
        
        TradingEngine {
            matching_engine,
            risk_manager,
            order_rx,
            order_tx,
            trade_rx,
            trade_tx,
            metrics,
        }
    }
    
    /// Gets a sender for submitting orders
    pub fn get_order_sender(&self) -> mpsc::Sender<Order> {
        self.order_tx.clone()
    }
    
    /// Gets a sender for publishing trades
    pub fn get_trade_sender(&self) -> mpsc::Sender<Trade> {
        self.trade_tx.clone()
    }
    
    /// Starts the trading engine
    pub async fn start(&mut self) {
        // Process orders in a loop
        while let Some(order) = self.order_rx.recv().await {
            // 1. Validate order with risk manager
            if let Err(e) = self.risk_manager.validate_order(&order).await {
                // Log error and continue
                log::error!("Order validation failed: {}", e);
                continue;
            }
            
            // 2. Process order with matching engine
            let engine = self.matching_engine.write().await;
            match engine.process_order(order) {
                Ok(trades) => {
                    // Publish trades
                    for trade in trades {
                        let _ = self.trade_tx.send(trade).await;
                    }
                },
                Err(e) => {
                    // Log error
                    log::error!("Order processing failed: {}", e);
                }
            }
        }
    }
    
    /// Initializes the trading engine with default markets
    pub async fn initialize(&mut self) {
        let mut engine = self.matching_engine.write().await;
        
        // Add common markets
        engine.add_market("BTC/USD");
        engine.add_market("ETH/USD");
        engine.add_market("BTC/USDT");
        engine.add_market("ETH/USDT");
        
        // Start background tasks
        self.start_background_tasks();
    }
    
    /// Starts background tasks for maintenance and monitoring
    fn start_background_tasks(&self) {
        // Clone the necessary references
        let metrics = Arc::clone(&self.metrics);
        
        // Spawn a task to periodically log metrics
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Log metrics
                metrics.log_metrics();
            }
        });
    }
}

// Main function to run the trading engine
pub async fn run(config: Config) -> Result<()> {
    // Create metrics collector
    let metrics = Arc::new(MetricsCollector::new(&config.metrics_prefix));

    // Create and initialize matching engine
    let matching_engine = Arc::new(RwLock::new(MatchingEngine::new(Arc::clone(&metrics))));
    
    // Create account repository (placeholder - implement with actual database)
    let account_repo = Arc::new(db::repositories::MockAccountRepository::new());
    
    // Create risk manager
    let risk_manager = Arc::new(RiskManager::new(account_repo, Arc::clone(&metrics)));
    
    // Create trading engine
    let mut trading_engine = TradingEngine::new(
        matching_engine, 
        Arc::clone(&risk_manager), 
        Arc::clone(&metrics)
    );
    
    // Initialize the trading engine
    trading_engine.initialize().await;
    
    // Start the trading engine
    trading_engine.start().await;
    
    Ok(())
}
