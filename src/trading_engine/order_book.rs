// src/trading_engine/order_book.rs

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use rust_decimal::Decimal;
use parking_lot::{RwLock as PLRwLock, Mutex};
use crossbeam::queue::SegQueue;
use rayon::prelude::*;

use crate::models::{Side, OrderId, Symbol, Price, Quantity, Timestamp, Order, Trade, OrderStatus, TimeInForce};

/// Represents order execution statistics for monitoring and analytics
#[derive(Debug, Clone, Default)]
pub struct OrderBookStats {
    /// The total number of orders processed
    pub orders_processed: usize,
    /// The total number of trades executed
    pub trades_executed: usize,
    /// The total volume traded
    pub volume_traded: Decimal,
    /// Maximum processing time in nanoseconds
    pub max_processing_time_ns: u64,
    /// Average processing time in nanoseconds
    pub avg_processing_time_ns: u64,
    /// Count of orders that were matched immediately
    pub immediate_match_count: usize,
    /// Count of orders that were added to the book
    pub book_addition_count: usize,
}

/// A price level in the order book
#[derive(Debug, Clone)]
struct PriceLevel {
    price: Price,
    orders: Vec<Arc<PLRwLock<Order>>>, // Use parking_lot for more efficient locking
    total_quantity: Quantity,
}

impl PriceLevel {
    fn new(price: Price) -> Self {
        PriceLevel {
            price,
            orders: Vec::with_capacity(64), // Pre-allocate vector to reduce reallocations
            total_quantity: Decimal::ZERO,
        }
    }
    
    fn add_order(&mut self, order: Arc<PLRwLock<Order>>) {
        let quantity = order.read().remaining_quantity();
        self.total_quantity += quantity;
        self.orders.push(order);
    }
    
    fn remove_order(&mut self, order_id: &OrderId) -> Option<Arc<PLRwLock<Order>>> {
        if let Some(index) = self.orders.iter().position(|o| o.read().id == *order_id) {
            let order = self.orders.swap_remove(index); // Use swap_remove for O(1) removal
            let quantity = order.read().remaining_quantity();
            self.total_quantity -= quantity;
            Some(order)
        } else {
            None
        }
    }
    
    fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
    
    /// Get total quantity at this price level
    fn quantity(&self) -> Quantity {
        self.total_quantity
    }
    
    /// Count orders at this price level
    fn order_count(&self) -> usize {
        self.orders.len()
    }
}

/// Event type for order book notifications
#[derive(Debug, Clone)]
pub enum OrderBookEvent {
    /// An order was added to the book
    OrderAdded(Arc<PLRwLock<Order>>),
    /// An order was removed from the book
    OrderRemoved(OrderId),
    /// A trade was executed
    TradeExecuted(Trade),
    /// The best bid changed
    BestBidChanged(Option<Price>),
    /// The best ask changed
    BestAskChanged(Option<Price>),
}

/// The order book for a trading pair with high-performance optimizations
#[derive(Debug)]
pub struct OrderBook {
    symbol: Symbol,
    bids: BTreeMap<Price, PriceLevel>,
    asks: BTreeMap<Price, PriceLevel>,
    orders: HashMap<OrderId, (Side, Price)>,
    best_bid: Option<Price>,
    best_ask: Option<Price>,
    last_update_time: Timestamp,
    stats: OrderBookStats,
    // Event queue for publishing order book events
    events: Arc<SegQueue<OrderBookEvent>>,
    // Thread-safe snapshot state for efficient reads
    snapshot_lock: Arc<PLRwLock<()>>,
}

impl OrderBook {
    pub fn new(symbol: Symbol) -> Self {
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::with_capacity(10000), // Pre-allocate to reduce rehashing
            best_bid: None,
            best_ask: None,
            last_update_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            stats: OrderBookStats::default(),
            events: Arc::new(SegQueue::new()),
            snapshot_lock: Arc::new(PLRwLock::new(())),
        }
    }
    
    /// Get a clone of the current order book statistics
    pub fn get_stats(&self) -> OrderBookStats {
        self.stats.clone()
    }
    
    /// Get an event receiver for subscribing to order book events
    pub fn subscribe(&self) -> Arc<SegQueue<OrderBookEvent>> {
        self.events.clone()
    }
    
    /// Add a limit order to the book
    pub fn add_order(&mut self, order: Arc<PLRwLock<Order>>) -> Result<(), String> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        let order_read = order.read();
        
        if order_read.symbol != self.symbol {
            return Err(format!("Symbol mismatch: expected {}, got {}", self.symbol, order_read.symbol));
        }
        
        let price = match order_read.price {
            Some(p) => p,
            None => return Err("Market orders should be handled separately".to_string()),
        };
        
        // Track original best bid/ask for change detection
        let original_best_bid = self.best_bid;
        let original_best_ask = self.best_ask;
        
        match order_read.side {
            Side::Buy => {
                let price_level = self.bids
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price));
                price_level.add_order(order.clone());
                self.orders.insert(order_read.id, (Side::Buy, price));
                
                // Update best bid if necessary
                if self.best_bid.is_none() || price > self.best_bid.unwrap() {
                    self.best_bid = Some(price);
                    if original_best_bid != self.best_bid {
                        self.events.push(OrderBookEvent::BestBidChanged(self.best_bid));
                    }
                }
            },
            Side::Sell => {
                let price_level = self.asks
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price));
                price_level.add_order(order.clone());
                self.orders.insert(order_read.id, (Side::Sell, price));
                
                // Update best ask if necessary
                if self.best_ask.is_none() || price < self.best_ask.unwrap() {
                    self.best_ask = Some(price);
                    if original_best_ask != self.best_ask {
                        self.events.push(OrderBookEvent::BestAskChanged(self.best_ask));
                    }
                }
            }
        }
        
        // Update book stats
        self.stats.book_addition_count += 1;
        self.stats.orders_processed += 1;
        
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Calculate processing time for metrics
        let processing_time = self.last_update_time - start_time;
        self.stats.max_processing_time_ns = self.stats.max_processing_time_ns.max(processing_time);
        self.stats.avg_processing_time_ns = 
            (self.stats.avg_processing_time_ns * (self.stats.orders_processed as u64 - 1) + processing_time) / 
            self.stats.orders_processed as u64;
            
        // Publish order added event
        self.events.push(OrderBookEvent::OrderAdded(order.clone()));
        
        Ok(())
    }
    
    /// Remove an order from the book
    pub fn remove_order(&mut self, order_id: &OrderId) -> Option<Arc<PLRwLock<Order>>> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Track original best bid/ask for change detection
        let original_best_bid = self.best_bid;
        let original_best_ask = self.best_ask;
        
        let result = if let Some((side, price)) = self.orders.remove(order_id) {
            match side {
                Side::Buy => {
                    if let Some(price_level) = self.bids.get_mut(&price) {
                        let order = price_level.remove_order(order_id);
                        if price_level.is_empty() {
                            self.bids.remove(&price);
                            
                            // Update best bid if necessary
                            if Some(price) == self.best_bid {
                                self.best_bid = self.bids.keys().next_back().cloned();
                                if original_best_bid != self.best_bid {
                                    self.events.push(OrderBookEvent::BestBidChanged(self.best_bid));
                                }
                            }
                        }
                        
                        self.last_update_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                            
                        // Publish order removed event
                        self.events.push(OrderBookEvent::OrderRemoved(*order_id));
                        
                        order
                    } else {
                        None
                    }
                },
                Side::Sell => {
                    if let Some(price_level) = self.asks.get_mut(&price) {
                        let order = price_level.remove_order(order_id);
                        if price_level.is_empty() {
                            self.asks.remove(&price);
                            
                            // Update best ask if necessary
                            if Some(price) == self.best_ask {
                                self.best_ask = self.asks.keys().next().cloned();
                                if original_best_ask != self.best_ask {
                                    self.events.push(OrderBookEvent::BestAskChanged(self.best_ask));
                                }
                            }
                        }
                        
                        self.last_update_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                            
                        // Publish order removed event
                        self.events.push(OrderBookEvent::OrderRemoved(*order_id));
                        
                        order
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };
        
        // Calculate processing time for metrics
        if result.is_some() {
            let processing_time = self.last_update_time - start_time;
            self.stats.max_processing_time_ns = self.stats.max_processing_time_ns.max(processing_time);
            
            self.stats.orders_processed += 1;
            self.stats.avg_processing_time_ns = 
                (self.stats.avg_processing_time_ns * (self.stats.orders_processed as u64 - 1) + processing_time) / 
                self.stats.orders_processed as u64;
        }
        
        result
    }
    
    /// Get the best bid price
    pub fn get_best_bid(&self) -> Option<Price> {
        self.best_bid
    }
    
    /// Get the best ask price
    pub fn get_best_ask(&self) -> Option<Price> {
        self.best_ask
    }
    
    /// Get the current spread (difference between best ask and best bid)
    pub fn get_spread(&self) -> Option<Decimal> {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
    
    /// Get current mid price ((best_bid + best_ask) / 2)
    pub fn get_mid_price(&self) -> Option<Decimal> {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => None,
        }
    }
    
    /// Get bid depth at specified levels
    pub fn get_bid_depth(&self, levels: usize) -> Vec<(Price, Quantity)> {
        self.bids.iter()
            .rev()
            .take(levels)
            .map(|(price, level)| (*price, level.total_quantity))
            .collect()
    }
    
    /// Get ask depth at specified levels
    pub fn get_ask_depth(&self, levels: usize) -> Vec<(Price, Quantity)> {
        self.asks.iter()
            .take(levels)
            .map(|(price, level)| (*price, level.total_quantity))
            .collect()
    }
    
    /// Get total volume at price or better
    pub fn get_volume_at_or_better(&self, side: Side, price: Price) -> Quantity {
        match side {
            Side::Buy => {
                self.bids.iter()
                    .rev()
                    .take_while(|(&bid_price, _)| bid_price >= price)
                    .fold(Decimal::ZERO, |acc, (_, level)| acc + level.total_quantity)
            },
            Side::Sell => {
                self.asks.iter()
                    .take_while(|(&ask_price, _)| ask_price <= price)
                    .fold(Decimal::ZERO, |acc, (_, level)| acc + level.total_quantity)
            }
        }
    }
    
    /// Match a limit order against the book and return resulting trades
    pub fn match_limit_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        let mut trades = Vec::new();
        
        // Cannot match if order has no price
        let order_price = match order.price {
            Some(price) => price,
            None => return trades,
        };
        
        // Track original best bid/ask for change detection
        let original_best_bid = self.best_bid;
        let original_best_ask = self.best_ask;
        
        match order.side {
            Side::Buy => {
                // Buy orders match against asks (sell orders)
                // Only match if the best ask exists and is <= order price
                while let Some(ask_price) = self.best_ask {
                    if ask_price > order_price || order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.asks.get_mut(&ask_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() && order.remaining_quantity() > Decimal::ZERO {
                            let maker_order = price_level.orders[i].clone();
                            let mut maker = maker_order.write();
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker.id,
                                    ask_price,
                                    trade_quantity,
                                    Side::Buy,
                                );
                                
                                trades.push(trade.clone());
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                maker.filled_quantity += trade_quantity;
                                
                                // Update price level's total quantity
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker.remaining_quantity() <= Decimal::ZERO {
                                    // Remove filled maker order
                                    maker.status = OrderStatus::Filled;
                                    price_level.orders.swap_remove(i);
                                    self.orders.remove(&maker.id);
                                    
                                    // Publish order removed event
                                    self.events.push(OrderBookEvent::OrderRemoved(maker.id));
                                } else {
                                    maker.status = OrderStatus::PartiallyFilled;
                                    i += 1;
                                }
                                
                                // Update trading stats
                                self.stats.trades_executed += 1;
                                self.stats.volume_traded += trade_quantity;
                                
                                // Publish trade event
                                self.events.push(OrderBookEvent::TradeExecuted(trade));
                            } else {
                                i += 1;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.asks.remove(&ask_price);
                            
                            // Update best ask
                            self.best_ask = self.asks.keys().next().cloned();
                            if original_best_ask != self.best_ask {
                                self.events.push(OrderBookEvent::BestAskChanged(self.best_ask));
                            }
                        }
                    } else {
                        break;
                    }
                }
            },
            Side::Sell => {
                // Sell orders match against bids (buy orders)
                // Only match if the best bid exists and is >= order price
                while let Some(bid_price) = self.best_bid {
                    if bid_price < order_price || order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.bids.get_mut(&bid_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() && order.remaining_quantity() > Decimal::ZERO {
                            let maker_order = price_level.orders[i].clone();
                            let mut maker = maker_order.write();
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker.id,
                                    bid_price,
                                    trade_quantity,
                                    Side::Sell,
                                );
                                
                                trades.push(trade.clone());
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                maker.filled_quantity += trade_quantity;
                                
                                // Update price level's total quantity
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker.remaining_quantity() <= Decimal::ZERO {
                                    // Remove filled maker order
                                    maker.status = OrderStatus::Filled;
                                    price_level.orders.swap_remove(i);
                                    self.orders.remove(&maker.id);
                                    
                                    // Publish order removed event
                                    self.events.push(OrderBookEvent::OrderRemoved(maker.id));
                                } else {
                                    maker.status = OrderStatus::PartiallyFilled;
                                    i += 1;
                                }
                                
                                // Update trading stats
                                self.stats.trades_executed += 1;
                                self.stats.volume_traded += trade_quantity;
                                
                                // Publish trade event
                                self.events.push(OrderBookEvent::TradeExecuted(trade));
                            } else {
                                i += 1;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.bids.remove(&bid_price);
                            
                            // Update best bid
                            self.best_bid = self.bids.keys().next_back().cloned();
                            if original_best_bid != self.best_bid {
                                self.events.push(OrderBookEvent::BestBidChanged(self.best_bid));
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        
        // Update order status
        if order.remaining_quantity() <= Decimal::ZERO {
            order.status = OrderStatus::Filled;
            self.stats.immediate_match_count += 1;
        } else if order.filled_quantity > Decimal::ZERO {
            order.status = OrderStatus::PartiallyFilled;
            
            // Add remainder to order book if not IOC or FOK
            match order.time_in_force {
                TimeInForce::ImmediateOrCancel => {
                    // IOC orders that aren't fully filled are partially canceled
                    order.status = OrderStatus::PartiallyFilled;
                },
                TimeInForce::FillOrKill => {
                    // FOK orders that aren't fully filled should be canceled
                    // But this shouldn't happen as we should check this before matching
                    order.status = OrderStatus::Canceled;
                    
                    // Cancel all trades (in a real system, we'd use a transaction)
                    trades.clear();
                    order.filled_quantity = Decimal::ZERO;
                },
                _ => {
                    // Only add to book if there's remaining quantity and it's not IOC/FOK
                    if order.remaining_quantity() > Decimal::ZERO && order.price.is_some() {
                        let order_arc = Arc::new(PLRwLock::new(order.clone()));
                        match self.add_order(order_arc) {
                            Ok(_) => {},
                            Err(e) => eprintln!("Error adding remainder to book: {}", e),
                        }
                    }
                }
            }
        }
        
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Calculate processing time for metrics
        let processing_time = self.last_update_time - start_time;
        self.stats.max_processing_time_ns = self.stats.max_processing_time_ns.max(processing_time);
        
        self.stats.orders_processed += 1;
        self.stats.avg_processing_time_ns = 
            (self.stats.avg_processing_time_ns * (self.stats.orders_processed as u64 - 1) + processing_time) / 
            self.stats.orders_processed as u64;
        
        trades
    }
    
    /// Match a market order against the book and return resulting trades
    pub fn match_market_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        let mut trades = Vec::new();
        
        // Track original best bid/ask for change detection
        let original_best_bid = self.best_bid;
        let original_best_ask = self.best_ask;
        
        match order.side {
            Side::Buy => {
                // Buy market orders match against asks (sell orders) at any price
                while let Some(ask_price) = self.best_ask {
                    if order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.asks.get_mut(&ask_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() && order.remaining_quantity() > Decimal::ZERO {
                            let maker_order = price_level.orders[i].clone();
                            let mut maker = maker_order.write();
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker.id,
                                    ask_price,
                                    trade_quantity,
                                    Side::Buy,
                                );
                                
                                trades.push(trade.clone());
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                maker.filled_quantity += trade_quantity;
                                
                                // Update price level's total quantity
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker.remaining_quantity() <= Decimal::ZERO {
                                    // Remove filled maker order
                                    maker.status = OrderStatus::Filled;
                                    price_level.orders.swap_remove(i);
                                    self.orders.remove(&maker.id);
                                    
                                    // Publish order removed event
                                    self.events.push(OrderBookEvent::OrderRemoved(maker.id));
                                } else {
                                    maker.status = OrderStatus::PartiallyFilled;
                                    i += 1;
                                }
                                
                                // Update trading stats
                                self.stats.trades_executed += 1;
                                self.stats.volume_traded += trade_quantity;
                                
                                // Publish trade event
                                self.events.push(OrderBookEvent::TradeExecuted(trade));
                            } else {
                                i += 1;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.asks.remove(&ask_price);
                            
                            // Update best ask
                            self.best_ask = self.asks.keys().next().cloned();
                            if original_best_ask != self.best_ask {
                                self.events.push(OrderBookEvent::BestAskChanged(self.best_ask));
                            }
                        }
                    } else {
                        break;
                    }
                }
            },
            Side::Sell => {
                // Sell market orders match against bids (buy orders) at any price
                while let Some(bid_price) = self.best_bid {
                    if order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.bids.get_mut(&bid_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() && order.remaining_quantity() > Decimal::ZERO {
                            let maker_order = price_level.orders[i].clone();
                            let mut maker = maker_order.write();
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker.id,
                                    bid_price,
                                    trade_quantity,
                                    Side::Sell,
                                );
                                
                                trades.push(trade.clone());
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                maker.filled_quantity += trade_quantity;
                                
                                // Update price level's total quantity
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker.remaining_quantity() <= Decimal::ZERO {
                                    // Remove filled maker order
                                    maker.status = OrderStatus::Filled;
                                    price_level.orders.swap_remove(i);
                                    self.orders.remove(&maker.id);
                                    
                                    // Publish order removed event
                                    self.events.push(OrderBookEvent::OrderRemoved(maker.id));
                                } else {
                                    maker.status = OrderStatus::PartiallyFilled;
                                    i += 1;
                                }
                                
                                // Update trading stats
                                self.stats.trades_executed += 1;
                                self.stats.volume_traded += trade_quantity;
                                
                                // Publish trade event
                                self.events.push(OrderBookEvent::TradeExecuted(trade));
                            } else {
                                i += 1;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.bids.remove(&bid_price);
                            
                            // Update best bid
                            self.best_bid = self.bids.keys().next_back().cloned();
                            if original_best_bid != self.best_bid {
                                self.events.push(OrderBookEvent::BestBidChanged(self.best_bid));
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        
        // Update order status
        if order.remaining_quantity() <= Decimal::ZERO {
            order.status = OrderStatus::Filled;
            self.stats.immediate_match_count += 1;
        } else if order.filled_quantity > Decimal::ZERO {
            order.status = OrderStatus::PartiallyFilled;
        } else {
            order.status = OrderStatus::Rejected; // Market orders that can't be filled are rejected
        }
        
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Calculate processing time for metrics
        let processing_time = self.last_update_time - start_time;
        self.stats.max_processing_time_ns = self.stats.max_processing_time_ns.max(processing_time);
        
        self.stats.orders_processed += 1;
        self.stats.avg_processing_time_ns = 
            (self.stats.avg_processing_time_ns * (self.stats.orders_processed as u64 - 1) + processing_time) / 
            self.stats.orders_processed as u64;
        
        trades
    }
    
    /// Process an iceberg order (large order divided into smaller visible portions)
    pub fn process_iceberg_order(&mut self, order: &mut Order, visible_quantity: Quantity) -> Vec<Trade> {
        if visible_quantity >= order.quantity {
            // If visible quantity is greater than total, process as normal order
            return match order.price {
                Some(_) => self.match_limit_order(order),
                None => self.match_market_order(order),
            };
        }
        
        let mut all_trades = Vec::new();
        let mut remaining = order.quantity;
        
        while remaining > Decimal::ZERO {
            // Create a "slice" of the iceberg order
            let mut slice = order.clone();
            slice.quantity = std::cmp::min(visible_quantity, remaining);
            slice.filled_quantity = Decimal::ZERO;
            
            // Match the slice
            let trades = match order.price {
                Some(_) => self.match_limit_order(&mut slice),
                None => self.match_market_order(&mut slice),
            };
            
            // Update the original order's filled quantity
            order.filled_quantity += slice.filled_quantity;
            remaining -= slice.filled_quantity;
            
            // Add trades to the result
            all_trades.extend(trades);
            
            // Stop if we couldn't match any more
            if slice.filled_quantity < slice.quantity {
                break;
            }
        }
        
        // Update order status
        if order.remaining_quantity() <= Decimal::ZERO {
            order.status = OrderStatus::Filled;
        } else if order.filled_quantity > Decimal::ZERO {
            order.status = OrderStatus::PartiallyFilled;
        }
        
        all_trades
    }
    
    /// Get a snapshot of the order book for a specific number of levels
    pub fn get_snapshot(&self, levels: usize) -> OrderBookSnapshot {
        // Use a read lock to ensure consistency during snapshot creation
        let _lock = self.snapshot_lock.read();
        
        OrderBookSnapshot {
            symbol: self.symbol.clone(),
            bids: self.get_bid_depth(levels),
            asks: self.get_ask_depth(levels),
            last_update_time: self.last_update_time,
        }
    }
    
    /// Get the total number of orders in the book
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }
    
    /// Get the total number of price levels in the book
    pub fn price_level_count(&self) -> usize {
        self.bids.len() + self.asks.len()
    }
    
    /// Clear the order book
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.orders.clear();
        self.best_bid = None;
        self.best_ask = None;
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
    }
    
    /// Get the time of the last update
    pub fn last_update_time(&self) -> Timestamp {
        self.last_update_time
    }
}

/// Represents a snapshot of the order book at a point in time
#[derive(Debug, Clone)]
pub struct OrderBookSnapshot {
    pub symbol: Symbol,
    pub bids: Vec<(Price, Quantity)>,
    pub asks: Vec<(Price, Quantity)>,
    pub last_update_time: Timestamp,
}

/// Implement unit tests for the order book
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimeInForce;
    
    fn create_order(
        id: &str, 
        side: Side, 
        price: Option<Decimal>, 
        quantity: Decimal,
        time_in_force: TimeInForce
    ) -> Order {
        Order {
            id: Uuid::parse_str(id).unwrap(),
            symbol: "BTC/USD".to_string(),
            side,
            order_type: if price.is_some() { "LIMIT".to_string() } else { "MARKET".to_string() },
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::New,
            time_in_force,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        }
    }
    
    #[test]
    fn test_add_and_remove_order() {
        let mut order_book = OrderBook::new("BTC/USD".to_string());
        
        // Create a buy order
        let mut buy_order = create_order(
            "00000000-0000-0000-0000-000000000001",
            Side::Buy,
            Some(Decimal::from(10000)),
            Decimal::from(1),
            TimeInForce::GoodTillCancel,
        );
        
        // Add the order to the book
        let order_arc = Arc::new(PLRwLock::new(buy_order.clone()));
        assert!(order_book.add_order(order_arc.clone()).is_ok());
        
        // Check best bid
        assert_eq!(order_book.get_best_bid(), Some(Decimal::from(10000)));
        
        // Remove the order
        let removed = order_book.remove_order(&buy_order.id);
        assert!(removed.is_some());
        
        // Check if best bid is gone
        assert_eq!(order_book.get_best_bid(), None);
    }
    
    #[test]
    fn test_match_limit_order() {
        let mut order_book = OrderBook::new("BTC/USD".to_string());
        
        // Add a sell order to the book
        let mut sell_order = create_order(
            "00000000-0000-0000-0000-000000000001",
            Side::Sell,
            Some(Decimal::from(10000)),
            Decimal::from(1),
            TimeInForce::GoodTillCancel,
        );
        
        let sell_arc = Arc::new(PLRwLock::new(sell_order.clone()));
        assert!(order_book.add_order(sell_arc.clone()).is_ok());
        
        // Create a buy order that matches
        let mut buy_order = create_order(
            "00000000-0000-0000-0000-000000000002",
            Side::Buy,
            Some(Decimal::from(10000)),
            Decimal::from(0.5),
            TimeInForce::GoodTillCancel,
        );
        
        // Match the order
        let trades = order_book.match_limit_order(&mut buy_order);
        
        // Verify trade details
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, buy_order.id);
        assert_eq!(trades[0].maker_order_id, sell_order.id);
        assert_eq!(trades[0].price, Decimal::from(10000));
        assert_eq!(trades[0].quantity, Decimal::from(0.5));
        
        // Verify order book state after matching
        assert_eq!(order_book.order_count(), 1); // Sell order still in book with 0.5 remaining
    }
    
    #[test]
    fn test_match_market_order() {
        let mut order_book = OrderBook::new("BTC/USD".to_string());
        
        // Add a sell order to the book
        let mut sell_order = create_order(
            "00000000-0000-0000-0000-000000000001",
            Side::Sell,
            Some(Decimal::from(10000)),
            Decimal::from(1),
            TimeInForce::GoodTillCancel,
        );
        
        let sell_arc = Arc::new(PLRwLock::new(sell_order.clone()));
        assert!(order_book.add_order(sell_arc.clone()).is_ok());
        
        // Create a market buy order
        let mut buy_order = create_order(
            "00000000-0000-0000-0000-000000000002",
            Side::Buy,
            None, // Market order has no price
            Decimal::from(0.5),
            TimeInForce::GoodTillCancel,
        );
        
        // Match the order
        let trades = order_book.match_market_order(&mut buy_order);
        
        // Verify trade details
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, buy_order.id);
        assert_eq!(trades[0].maker_order_id, sell_order.id);
        assert_eq!(trades[0].price, Decimal::from(10000));
        assert_eq!(trades[0].quantity, Decimal::from(0.5));
        
        // Verify market order status
        assert_eq!(buy_order.status, OrderStatus::Filled);
    }
}
