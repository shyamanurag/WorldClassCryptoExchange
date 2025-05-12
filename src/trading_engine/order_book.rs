
use std::collections::{BTreeMap, HashMap};
use std::cmp::Ordering;
use crate::models::{Order, OrderSide, OrderType, OrderStatus};

pub struct OrderBook {
    symbol: String,
    bids: BTreeMap<u64, Vec<Order>>, // Price -> Orders (price in minor units to avoid float issues)
    asks: BTreeMap<u64, Vec<Order>>, // Price -> Orders
    order_map: HashMap<String, (u64, OrderSide)>, // OrderID -> (Price, Side) for quick lookups
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_map: HashMap::new(),
        }
    }
    
    // Add a limit order to the book
    pub fn add_limit_order(&mut self, order: Order) -> Result<(), String> {
        // Validate order
        if order.order_type != OrderType::Limit {
            return Err("Only limit orders can be added to the order book".to_string());
        }
        
        let price_key = (order.price * 10000.0) as u64; // Convert to integer (adjust scale as needed)
        
        match order.side {
            OrderSide::Buy => {
                self.bids.entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(order.clone());
                // Sort by time for price-time priority
                if let Some(orders) = self.bids.get_mut(&price_key) {
                    orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                }
            },
            OrderSide::Sell => {
                self.asks.entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(order.clone());
                if let Some(orders) = self.asks.get_mut(&price_key) {
                    orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                }
            }
        }
        
        // Add to order map for quick lookups
        self.order_map.insert(order.id.clone(), (price_key, order.side));
        
        Ok(())
    }
    
    // Remove an order from the book
    pub fn remove_order(&mut self, order_id: &str) -> Result<Order, String> {
        // Implementation details
    }
    
    // Get best bid price
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.keys().next_back().map(|&price| price as f64 / 10000.0)
    }
    
    // Get best ask price
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|&price| price as f64 / 10000.0)
    }
    
    // Get market depth at specific levels
    pub fn get_depth(&self, levels: usize) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        // Implementation details
    }
}
