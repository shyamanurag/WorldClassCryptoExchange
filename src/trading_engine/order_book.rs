rustuse std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

use crate::models::{Order, OrderSide, OrderStatus, OrderType, Trade};
use crate::utils::metrics::MetricsCollector;

/// The OrderBook holds buy and sell orders for a specific trading pair.
/// It implements price-time priority ordering and efficient matching.
pub struct OrderBook {
    /// Trading pair symbol (e.g., "BTC/USD")
    symbol: String,
    
    /// Price levels for buy orders (bids), sorted in descending order
    /// Key: Price in minor units (satoshis, cents, etc.)
    /// Value: Orders at this price level
    bids: BTreeMap<u64, VecDeque<Order>>,
    
    /// Price levels for sell orders (asks), sorted in ascending order
    /// Key: Price in minor units
    /// Value: Orders at this price level
    asks: BTreeMap<u64, VecDeque<Order>>,
    
    /// Quick lookup map for finding orders by ID
    /// Key: Order ID
    /// Value: (Price, Side) tuple
    order_map: HashMap<String, (u64, OrderSide)>,
    
    /// Total volume at each price level (for quick market depth calculations)
    /// Key: Price in minor units
    /// Value: Total volume at this price
    bid_volumes: HashMap<u64, f64>,
    ask_volumes: HashMap<u64, f64>,
    
    /// Broadcast channel for order book updates
    update_sender: broadcast::Sender<OrderBookUpdate>,
    
    /// Metrics collector for performance monitoring
    metrics: Arc<MetricsCollector>,
    
    /// Last update timestamp
    last_update: u64,
}

/// Represents an update to the order book
#[derive(Clone, Debug)]
pub struct OrderBookUpdate {
    pub symbol: String,
    pub timestamp: u64,
    pub bids: Vec<(f64, f64)>, // (price, quantity)
    pub asks: Vec<(f64, f64)>, // (price, quantity)
}

impl OrderBook {
    /// Creates a new order book for the specified symbol
    pub fn new(symbol: String, metrics: Arc<MetricsCollector>) -> Self {
        let (update_sender, _) = broadcast::channel(100);
        
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_map: HashMap::new(),
            bid_volumes: HashMap::new(),
            ask_volumes: HashMap::new(),
            update_sender,
            metrics,
            last_update: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
    
    /// Adds a limit order to the order book
    pub fn add_limit_order(&mut self, order: Order) -> Result<(), String> {
        let start = Instant::now();
        
        // Validate order
        if order.order_type != OrderType::Limit {
            return Err("Only limit orders can be added to the order book".to_string());
        }
        
        if order.remaining_quantity <= 0.0 {
            return Err("Order quantity must be greater than zero".to_string());
        }
        
        // Convert price to integer to avoid floating-point issues
        // We multiply by a scaling factor (e.g., 10000 for 4 decimal places)
        let price_key = (order.price * 10000.0) as u64;
        
        match order.side {
            OrderSide::Buy => {
                // Add to bids (buy orders)
                self.bids.entry(price_key)
                    .or_insert_with(VecDeque::new)
                    .push_back(order.clone());
                
                // Update volume tracking
                *self.bid_volumes.entry(price_key).or_insert(0.0) += order.remaining_quantity;
            },
            OrderSide::Sell => {
                // Add to asks (sell orders)
                self.asks.entry(price_key)
                    .or_insert_with(VecDeque::new)
                    .push_back(order.clone());
                
                // Update volume tracking
                *self.ask_volumes.entry(price_key).or_insert(0.0) += order.remaining_quantity;
            }
        }
        
        // Add to order map for quick lookups
        self.order_map.insert(order.id.clone(), (price_key, order.side));
        
        // Update timestamp
        self.last_update = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // Broadcast order book update
        let _ = self.broadcast_update();
        
        // Record metrics
        let elapsed = start.elapsed().as_micros() as u64;
        self.metrics.record_order_book_operation_time("add_limit_order", &self.symbol, elapsed);
        
        Ok(())
    }
    
    /// Removes an order from the order book
    pub fn remove_order(&mut self, order_id: &str) -> Result<Order, String> {
        let start = Instant::now();
        
        // Find order in the map
        let (price_key, side) = self.order_map.get(order_id)
            .ok_or_else(|| format!("Order {} not found", order_id))?;
        
        let price_key = *price_key;
        let side = *side;
        
        // Remove from the appropriate order list
        let orders = match side {
            OrderSide::Buy => self.bids.get_mut(&price_key),
            OrderSide::Sell => self.asks.get_mut(&price_key),
        };
        
        let mut removed_order = None;
        
        if let Some(orders) = orders {
            // Find and remove the order
            let position = orders.iter().position(|o| o.id == order_id);
            
            if let Some(pos) = position {
                let order = orders.remove(pos).unwrap();
                removed_order = Some(order.clone());
                
                // Update volume tracking
                match side {
                    OrderSide::Buy => {
                        if let Some(volume) = self.bid_volumes.get_mut(&price_key) {
                            *volume -= order.remaining_quantity;
                            if *volume <= 0.0 {
                                self.bid_volumes.remove(&price_key);
                                // Remove empty price level
                                if orders.is_empty() {
                                    self.bids.remove(&price_key);
                                }
                            }
                        }
                    },
                    OrderSide::Sell => {
                        if let Some(volume) = self.ask_volumes.get_mut(&price_key) {
                            *volume -= order.remaining_quantity;
                            if *volume <= 0.0 {
                                self.ask_volumes.remove(&price_key);
                                // Remove empty price level
                                if orders.is_empty() {
                                    self.asks.remove(&price_key);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Remove from the order map
        self.order_map.remove(order_id);
        
        // Update timestamp
        self.last_update = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // Broadcast order book update
        let _ = self.broadcast_update();
        
        // Record metrics
        let elapsed = start.elapsed().as_micros() as u64;
        self.metrics.record_order_book_operation_time("remove_order", &self.symbol, elapsed);
        
        match removed_order {
            Some(order) => Ok(order),
            None => Err(format!("Order {} not found in order list", order_id)),
        }
    }
    
    /// Updates an order in the order book (partial fill or price change)
    pub fn update_order(&mut self, order_id: &str, new_quantity: f64, new_price: Option<f64>) -> Result<Order, String> {
        let start = Instant::now();
        
        // Remove the old order
        let mut order = self.remove_order(order_id)?;
        
        // Update the order
        order.remaining_quantity = new_quantity;
        if let Some(price) = new_price {
            order.price = price;
        }
        
        // Add the updated order back to the book
        self.add_limit_order(order.clone())?;
        
        // Record metrics
        let elapsed = start.elapsed().as_micros() as u64;
        self.metrics.record_order_book_operation_time("update_order", &self.symbol, elapsed);
        
        Ok(order)
    }
    
    /// Gets the best bid price
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.keys().next_back().map(|&price| price as f64 / 10000.0)
    }
    
    /// Gets the best ask price
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|&price| price as f64 / 10000.0)
    }
    
    /// Gets the current spread
    pub fn spread(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask - bid),
            _ => None,
        }
    }
    
    /// Gets market depth at specific levels
    pub fn get_depth(&self, levels: usize) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let mut bids = Vec::with_capacity(levels);
        let mut asks = Vec::with_capacity(levels);
        
        // Get top bid levels (sorted in descending order)
        for (&price, volume) in self.bid_volumes.iter()
            .filter(|(_, &volume)| volume > 0.0)
            .take(levels)
        {
            bids.push((price as f64 / 10000.0, *volume));
        }
        
        // Sort bids by price (descending)
        bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // Get top ask levels (sorted in ascending order)
        for (&price, volume) in self.ask_volumes.iter()
            .filter(|(_, &volume)| volume > 0.0)
            .take(levels)
        {
            asks.push((price as f64 / 10000.0, *volume));
        }
        
        // Sort asks by price (ascending)
        asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        
        (bids, asks)
    }
    
    /// Gets a subscription to order book updates
    pub fn subscribe(&self) -> broadcast::Receiver<OrderBookUpdate> {
        self.update_sender.subscribe()
    }
    
    /// Broadcasts an order book update
    fn broadcast_update(&self) -> Result<(), String> {
        let (bids, asks) = self.get_depth(10); // Top 10 levels
        
        let update = OrderBookUpdate {
            symbol: self.symbol.clone(),
            timestamp: self.last_update,
            bids,
            asks,
        };
        
        if self.update_sender.receiver_count() > 0 {
            if let Err(e) = self.update_sender.send(update) {
                return Err(format!("Failed to broadcast order book update: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Calculates the average fill price for a market order of the given size
    pub fn calculate_market_price(&self, side: OrderSide, quantity: f64) -> Option<f64> {
        let mut remaining = quantity;
        let mut total_cost = 0.0;
        
        match side {
            OrderSide::Buy => {
                // For buy orders, we need to match against the asks (sell orders)
                for (&price, &volume) in self.ask_volumes.iter()
                    .filter(|(_, &volume)| volume > 0.0)
                {
                    let price_f64 = price as f64 / 10000.0;
                    let fill_qty = remaining.min(volume);
                    total_cost += fill_qty * price_f64;
                    remaining -= fill_qty;
                    
                    if remaining <= 0.0 {
                        break;
                    }
                }
            },
            OrderSide::Sell => {
                // For sell orders, we need to match against the bids (buy orders)
                for (&price, &volume) in self.bid_volumes.iter()
                    .filter(|(_, &volume)| volume > 0.0)
                    .rev() // Descending order for bids
                {
                    let price_f64 = price as f64 / 10000.0;
                    let fill_qty = remaining.min(volume);
                    total_cost += fill_qty * price_f64;
                    remaining -= fill_qty;
                    
                    if remaining <= 0.0 {
                        break;
                    }
                }
            }
        }
        
        if remaining < quantity {
            // At least some of the order would be filled
            let filled = quantity - remaining;
            Some(total_cost / filled)
        } else {
            // No liquidity to fill the order
            None
        }
    }
}
