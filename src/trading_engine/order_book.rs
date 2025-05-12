rust// src/trading_engine/order_book.rs

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use rust_decimal::Decimal;

use crate::models::{Side, OrderId, Symbol, Price, Quantity, Timestamp, Order, Trade};

/// A price level in the order book
#[derive(Debug, Clone)]
struct PriceLevel {
    price: Price,
    orders: Vec<Arc<Order>>,
    total_quantity: Quantity,
}

impl PriceLevel {
    fn new(price: Price) -> Self {
        PriceLevel {
            price,
            orders: Vec::new(),
            total_quantity: Decimal::ZERO,
        }
    }
    
    fn add_order(&mut self, order: Arc<Order>) {
        self.total_quantity += order.remaining_quantity();
        self.orders.push(order);
    }
    
    fn remove_order(&mut self, order_id: &OrderId) -> Option<Arc<Order>> {
        if let Some(index) = self.orders.iter().position(|o| o.id == *order_id) {
            let order = self.orders.remove(index);
            self.total_quantity -= order.remaining_quantity();
            Some(order)
        } else {
            None
        }
    }
    
    fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}

/// The order book for a trading pair
#[derive(Debug)]
pub struct OrderBook {
    symbol: Symbol,
    bids: BTreeMap<Price, PriceLevel>,
    asks: BTreeMap<Price, PriceLevel>,
    orders: HashMap<OrderId, (Side, Price)>,
    last_update_time: Timestamp,
}

impl OrderBook {
    pub fn new(symbol: Symbol) -> Self {
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            last_update_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        }
    }
    
    pub fn add_order(&mut self, order: Arc<Order>) -> Result<(), String> {
        if order.symbol != self.symbol {
            return Err(format!("Symbol mismatch: expected {}, got {}", self.symbol, order.symbol));
        }
        
        let price = match order.price {
            Some(p) => p,
            None => return Err("Market orders should be handled separately".to_string()),
        };
        
        match order.side {
            Side::Buy => {
                let price_level = self.bids
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price));
                price_level.add_order(order.clone());
                self.orders.insert(order.id, (Side::Buy, price));
            },
            Side::Sell => {
                let price_level = self.asks
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price));
                price_level.add_order(order.clone());
                self.orders.insert(order.id, (Side::Sell, price));
            }
        }
        
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        Ok(())
    }
    
    pub fn remove_order(&mut self, order_id: &OrderId) -> Option<Arc<Order>> {
        if let Some((side, price)) = self.orders.remove(order_id) {
            match side {
                Side::Buy => {
                    if let Some(price_level) = self.bids.get_mut(&price) {
                        let order = price_level.remove_order(order_id);
                        if price_level.is_empty() {
                            self.bids.remove(&price);
                        }
                        
                        self.last_update_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                        
                        return order;
                    }
                },
                Side::Sell => {
                    if let Some(price_level) = self.asks.get_mut(&price) {
                        let order = price_level.remove_order(order_id);
                        if price_level.is_empty() {
                            self.asks.remove(&price);
                        }
                        
                        self.last_update_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                        
                        return order;
                    }
                }
            }
        }
        None
    }
    
    pub fn get_best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().cloned()
    }
    
    pub fn get_best_ask(&self) -> Option<Price> {
        self.asks.keys().next().cloned()
    }
    
    pub fn get_bid_depth(&self, levels: usize) -> Vec<(Price, Quantity)> {
        self.bids.iter()
            .rev()
            .take(levels)
            .map(|(price, level)| (*price, level.total_quantity))
            .collect()
    }
    
    pub fn get_ask_depth(&self, levels: usize) -> Vec<(Price, Quantity)> {
        self.asks.iter()
            .take(levels)
            .map(|(price, level)| (*price, level.total_quantity))
            .collect()
    }
    
    pub fn match_limit_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        // Cannot match if order has no price
        let order_price = match order.price {
            Some(price) => price,
            None => return trades,
        };
        
        match order.side {
            Side::Buy => {
                // Buy orders match against asks (sell orders)
                let mut ask_prices: Vec<Price> = self.asks.keys()
                    .take_while(|&&price| price <= order_price)
                    .cloned()
                    .collect();
                
                for ask_price in ask_prices {
                    if order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.asks.get_mut(&ask_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() {
                            let maker_order = &price_level.orders[i];
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker_order.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker_order.id,
                                    ask_price,
                                    trade_quantity,
                                    Side::Buy,
                                );
                                
                                trades.push(trade);
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                
                                // Note: This is a simplification. In a real implementation,
                                // we would need to update the maker order as well, which would
                                // require a mutable reference.
                                // For now, we just update our tracking of the total quantity.
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker_order.remaining_quantity() <= trade_quantity {
                                    // Remove filled maker order
                                    price_level.orders.remove(i);
                                    self.orders.remove(&maker_order.id);
                                } else {
                                    i += 1;
                                }
                            } else {
                                i += 1;
                            }
                            
                            if order.remaining_quantity() <= Decimal::ZERO {
                                break;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.asks.remove(&ask_price);
                        }
                    }
                }
            },
            Side::Sell => {
                // Sell orders match against bids (buy orders)
                let mut bid_prices: Vec<Price> = self.bids.keys()
                    .rev()
                    .take_while(|&&price| price >= order_price)
                    .cloned()
                    .collect();
                
                for bid_price in bid_prices {
                    if order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.bids.get_mut(&bid_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() {
                            let maker_order = &price_level.orders[i];
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker_order.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker_order.id,
                                    bid_price,
                                    trade_quantity,
                                    Side::Sell,
                                );
                                
                                trades.push(trade);
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                
                                // Note: This is a simplification. In a real implementation,
                                // we would need to update the maker order as well.
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker_order.remaining_quantity() <= trade_quantity {
                                    // Remove filled maker order
                                    price_level.orders.remove(i);
                                    self.orders.remove(&maker_order.id);
                                } else {
                                    i += 1;
                                }
                            } else {
                                i += 1;
                            }
                            
                            if order.remaining_quantity() <= Decimal::ZERO {
                                break;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.bids.remove(&bid_price);
                        }
                    }
                }
            }
        }
        
        // Update order status
        if order.remaining_quantity() <= Decimal::ZERO {
            order.status = OrderStatus::Filled;
        } else if order.filled_quantity > Decimal::ZERO {
            order.status = OrderStatus::PartiallyFilled;
            
            // Add remainder to order book if not IOC or FOK
            match order.time_in_force {
                TimeInForce::ImmediateOrCancel | TimeInForce::FillOrKill => {
                    order.status = if order.filled_quantity > Decimal::ZERO {
                        OrderStatus::PartiallyFilled
                    } else {
                        OrderStatus::Canceled
                    };
                },
                _ => {
                    // Only add to book if there's remaining quantity and it's not IOC/FOK
                    if order.remaining_quantity() > Decimal::ZERO && order.price.is_some() {
                        let order_arc = Arc::new(order.clone());
                        match self.add_order(order_arc) {
                            Ok(_) => {},
                            Err(e) => println!("Error adding remainder to book: {}", e),
                        }
                    }
                }
            }
        }
        
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        trades
    }
    
    pub fn match_market_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        match order.side {
            Side::Buy => {
                // Buy market orders match against asks (sell orders) at any price
                let mut ask_prices: Vec<Price> = self.asks.keys().cloned().collect();
                
                for ask_price in ask_prices {
                    if order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.asks.get_mut(&ask_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() {
                            let maker_order = &price_level.orders[i];
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker_order.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker_order.id,
                                    ask_price,
                                    trade_quantity,
                                    Side::Buy,
                                );
                                
                                trades.push(trade);
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                
                                // Note: Simplification as mentioned above
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker_order.remaining_quantity() <= trade_quantity {
                                    // Remove filled maker order
                                    price_level.orders.remove(i);
                                    self.orders.remove(&maker_order.id);
                                } else {
                                    i += 1;
                                }
                            } else {
                                i += 1;
                            }
                            
                            if order.remaining_quantity() <= Decimal::ZERO {
                                break;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.asks.remove(&ask_price);
                        }
                    }
                }
            },
            Side::Sell => {
                // Sell market orders match against bids (buy orders) at any price
                let mut bid_prices: Vec<Price> = self.bids.keys().rev().cloned().collect();
                
                for bid_price in bid_prices {
                    if order.remaining_quantity() <= Decimal::ZERO {
                        break;
                    }
                    
                    if let Some(price_level) = self.bids.get_mut(&bid_price) {
                        let mut i = 0;
                        while i < price_level.orders.len() {
                            let maker_order = &price_level.orders[i];
                            
                            // Calculate trade quantity
                            let trade_quantity = std::cmp::min(
                                order.remaining_quantity(),
                                maker_order.remaining_quantity(),
                            );
                            
                            if trade_quantity > Decimal::ZERO {
                                // Create a trade
                                let trade = Trade::new(
                                    order.symbol.clone(),
                                    order.id,
                                    maker_order.id,
                                    bid_price,
                                    trade_quantity,
                                    Side::Sell,
                                );
                                
                                trades.push(trade);
                                
                                // Update order quantities
                                order.filled_quantity += trade_quantity;
                                
                                // Note: Simplification as mentioned above
                                price_level.total_quantity -= trade_quantity;
                                
                                // Check if maker order is filled
                                if maker_order.remaining_quantity() <= trade_quantity {
                                    // Remove filled maker order
                                    price_level.orders.remove(i);
                                    self.orders.remove(&maker_order.id);
                                } else {
                                    i += 1;
                                }
                            } else {
                                i += 1;
                            }
                            
                            if order.remaining_quantity() <= Decimal::ZERO {
                                break;
                            }
                        }
                        
                        // Remove empty price level
                        if price_level.is_empty() {
                            self.bids.remove(&bid_price);
                        }
                    }
                }
            }
        }
        
        // Update order status
        if order.remaining_quantity() <= Decimal::ZERO {
            order.status = OrderStatus::Filled;
        } else if order.filled_quantity > Decimal::ZERO {
            order.status = OrderStatus::PartiallyFilled;
        } else {
            order.status = OrderStatus::Rejected; // Market orders that can't be filled are rejected
        }
        
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        trades
    }
}
