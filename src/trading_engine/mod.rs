
pub mod order_book;
pub mod matching_engine;
pub mod risk_management;
pub mod market_data;

use tokio::sync::mpsc;
use crate::models::{Order, Trade};
use crate::metrics::MetricsCollector;

pub struct TradingEngine {
    // Components
    matching_engine: matching_engine::MatchingEngine,
    risk_manager: risk_management::RiskManager,
    
    // Channels
    order_tx: mpsc::Sender<Order>,
    order_rx: mpsc::Receiver<Order>,
    trade_tx: mpsc::Sender<Trade>,
    trade_rx: mpsc::Receiver<Trade>,
    
    // Metrics
    metrics: MetricsCollector,
}

impl TradingEngine {
    pub async fn start(&mut self) {
        // Start processing loop
        while let Some(order) = self.order_rx.recv().await {
            // 1. Risk check
            if let Err(e) = self.risk_manager.validate_order(&order, &order.user_id) {
                // Log and reject order
                continue;
            }
            
            // 2. Process order
            let trades = self.matching_engine.process_order(order);
            
            // 3. Publish trades
            for trade in trades {
                let _ = self.trade_tx.send(trade).await;
            }
        }
    }
}
