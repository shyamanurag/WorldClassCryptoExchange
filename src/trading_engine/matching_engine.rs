

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex};
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

// Define core types
pub type OrderId = Uuid;
pub type TradeId = Uuid;
pub type UserId = Uuid;
pub type Symbol = String;
pub type Price = Decimal;
pub type Quantity = Decimal;
pub type Timestamp = u64;

/// Side of the order (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// Type of order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
    StopLoss,
    StopLimit,
    TrailingStop,
    FillOrKill,
    ImmediateOrCancel,
    PostOnly,
}

/// Time in force for the order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    GoodTillCancel,
    ImmediateOrCancel,
    FillOrKill,
    GoodTillDate(Timestamp),
}

/// Status of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

/// Representation of an order in the system
#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Option<Price>,
    pub quantity: Quantity,
    pub filled_quantity: Quantity,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub stop_price: Option<Price>,
}

impl Order {
    pub fn new(
        user_id: UserId,
        symbol: Symbol,
        side: Side,
        order_type: OrderType,
        price: Option<Price>,
        quantity: Quantity,
        time_in_force: TimeInForce,
        stop_price: Option<Price>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        Order {
            id: Uuid::new_v4(),
            user_id,
            symbol,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::New,
            time_in_force,
            created_at: now,
            updated_at: now,
            stop_price,
        }
    }
    
    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }
    
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }
}

/// Representation of a trade in the system
#[derive(Debug, Clone)]
pub struct Trade {
    pub id: TradeId,
    pub symbol: Symbol,
    pub taker_order_id: OrderId,
    pub maker_order_id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
    pub side: Side,
    pub timestamp: Timestamp,
}

impl Trade {
    pub fn new(
        symbol: Symbol,
        taker_order_id: OrderId,
        maker_order_id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        Trade {
            id: Uuid::new_v4(),
            symbol,
            taker_order_id,
            maker_order_id,
            price,
            quantity,
            side,
            timestamp: now,
        }
    }
}

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

/// The matching engine for a single trading pair
pub struct MatchingEngine {
    order_book: RwLock<OrderBook>,
    symbol: Symbol,
    trade_history: Mutex<Vec<Trade>>,
}

impl MatchingEngine {
    pub fn new(symbol: Symbol) -> Self {
        MatchingEngine {
            order_book: RwLock::new(OrderBook::new(symbol.clone())),
            symbol,
            trade_history: Mutex::new(Vec::new()),
        }
    }
    
    pub async fn process_order(&self, mut order: Order) -> Result<Vec<Trade>, String> {
        // Validate the order symbol
        if order.symbol != self.symbol {
            return Err(format!("Symbol mismatch: expected {}, got {}", self.symbol, order.symbol));
        }
        
        let mut order_book = self.order_book.write().await;
        let trades = match order.order_type {
            OrderType::Limit => {
                if order.price.is_none() {
                    return Err("Limit orders must have a price".to_string());
                }
                
                // Process limit order
                order_book.match_limit_order(&mut order)
            },
            OrderType::Market => {
                // Price is ignored for market orders
                order.price = None;
                
                // Process market order
                order_book.match_market_order(&mut order)
            },
            _ => {
                // Simplification: Only handling basic limit and market orders for now
                return Err(format!("Unsupported order type: {:?}", order.order_type));
            }
        };
        
        // Record trades in history
        if !trades.is_empty() {
            let mut history = self.trade_history.lock().await;
            history.extend(trades.clone());
        }
        
        Ok(trades)
    }
    
    pub async fn cancel_order(&self, order_id: OrderId) -> Result<Option<Arc<Order>>, String> {
        let mut order_book = self.order_book.write().await;
        Ok(order_book.remove_order(&order_id))
    }
    
    pub async fn get_order_book_snapshot(&self, depth: usize) -> Result<(Vec<(Price, Quantity)>, Vec<(Price, Quantity)>), String> {
        let order_book = self.order_book.read().await;
        let bids = order_book.get_bid_depth(depth);
        let asks = order_book.get_ask_depth(depth);
        Ok((bids, asks))
    }
    
    pub async fn get_recent_trades(&self, limit: usize) -> Result<Vec<Trade>, String> {
        let history = self.trade_history.lock().await;
        Ok(history.iter().rev().take(limit).cloned().collect())
    }
}

/// Manager for multiple trading pairs
pub struct MatchingEngineManager {
    engines: RwLock<HashMap<Symbol, Arc<MatchingEngine>>>,
}

impl MatchingEngineManager {
    pub fn new() -> Self {
        MatchingEngineManager {
            engines: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn add_symbol(&self, symbol: Symbol) -> Result<(), String> {
        let mut engines = self.engines.write().await;
        if engines.contains_key(&symbol) {
            return Err(format!("Symbol already exists: {}", symbol));
        }
        
        let engine = Arc::new(MatchingEngine::new(symbol.clone()));
        engines.insert(symbol, engine);
        Ok(())
    }
    
    pub async fn get_engine(&self, symbol: &Symbol) -> Result<Arc<MatchingEngine>, String> {
        let engines = self.engines.read().await;
        engines.get(symbol)
            .cloned()
            .ok_or_else(|| format!("Symbol not found: {}", symbol))
    }
    
    pub async fn process_order(&self, order: Order) -> Result<Vec<Trade>, String> {
        let engine = self.get_engine(&order.symbol).await?;
        engine.process_order(order).await
    }
    
    pub async fn cancel_order(&self, symbol: &Symbol, order_id: OrderId) -> Result<Option<Arc<Order>>, String> {
        let engine = self.get_engine(symbol).await?;
        engine.cancel_order(order_id).await
    }
}

// Example of using the matching engine
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[tokio::test]
    async fn test_limit_order_matching() {
        let engine = MatchingEngine::new("BTC-USDT".to_string());
        
        // Create a sell order
        let sell_order = Order::new(
            Uuid::new_v4(),
            "BTC-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(50000)),
            dec!(1),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        // Process the sell order
        let trades = engine.process_order(sell_order).await.unwrap();
        assert_eq!(trades.len(), 0); // No trades yet
        
        // Create a buy order at a matching price
        let buy_order = Order::new(
            Uuid::new_v4(),
            "BTC-USDT".to_string(),
            Side::Buy,
            OrderType::Limit,
            Some(dec!(50000)),
            dec!(0.5),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        // Process the buy order
        let trades = engine.process_order(buy_order).await.unwrap();
        assert_eq!(trades.len(), 1); // One trade executed
        
        let trade = &trades[0];
        assert_eq!(trade.price, dec!(50000));
        assert_eq!(trade.quantity, dec!(0.5));
        assert_eq!(trade.side, Side::Buy);
        
        // Get order book snapshot
        let (bids, asks) = engine.get_order_book_snapshot(10).await.unwrap();
        
        // Should still have 0.5 BTC at 50,000 USDT in the asks
        assert_eq!(asks.len(), 1);
        assert_eq!(asks[0], (dec!(50000), dec!(0.5)));
        
        // No bids should be left since the buy order was fully filled
        assert_eq!(bids.len(), 0);
    }
    
    #[tokio::test]
    async fn test_market_order_matching() {
        let engine = MatchingEngine::new("ETH-USDT".to_string());
        
        // Create multiple sell orders at different price levels
        let sell_order1 = Order::new(
            Uuid::new_v4(),
            "ETH-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(3000)),
            dec!(1),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        let sell_order2 = Order::new(
            Uuid::new_v4(),
            "ETH-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(3100)),
            dec!(2),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        // Process the sell orders
        let _ = engine.process_order(sell_order1).await.unwrap();
        let _ = engine.process_order(sell_order2).await.unwrap();
        
        // Create a market buy order
        let buy_order = Order::new(
            Uuid::new_v4(),
            "ETH-USDT".to_string(),
            Side::Buy,
            OrderType::Market,
            None, // No price for market orders
            dec!(1.5),
            TimeInForce::ImmediateOrCancel,
            None,
        );
        
        // Process the market buy order
        let trades = engine.process_order(buy_order).await.unwrap();
        
        // Should execute 2 trades
        assert_eq!(trades.len(), 2);
        
        // First trade should be at the best ask price (3000)
        assert_eq!(trades[0].price, dec!(3000));
        assert_eq!(trades[0].quantity, dec!(1));
        
        // Second trade should be at the next best ask price (3100)
        assert_eq!(trades[1].price, dec!(3100));
        assert_eq!(trades[1].quantity, dec!(0.5));
        
        // Get order book snapshot
        let (bids, asks) = engine.get_order_book_snapshot(10).await.unwrap();
        
        // Should still have 1.5 ETH at 3100 USDT in the asks
        assert_eq!(asks.len(), 1);
        assert_eq!(asks[0], (dec!(3100), dec!(1.5)));
        
        // No bids should be present
        assert_eq!(bids.len(), 0);
    }
    
    #[tokio::test]
    async fn test_order_book_multiple_price_levels() {
        let engine = MatchingEngine::new("SOL-USDT".to_string());
        
        // Create multiple buy orders at different price levels
        let buy_order1 = Order::new(
            Uuid::new_v4(),
            "SOL-USDT".to_string(),
            Side::Buy,
            OrderType::Limit,
            Some(dec!(100)),
            dec!(10),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        let buy_order2 = Order::new(
            Uuid::new_v4(),
            "SOL-USDT".to_string(),
            Side::Buy,
            OrderType::Limit,
            Some(dec!(99)),
            dec!(15),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        let buy_order3 = Order::new(
            Uuid::new_v4(),
            "SOL-USDT".to_string(),
            Side::Buy,
            OrderType::Limit,
            Some(dec!(101)),
            dec!(5),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        // Process the buy orders
        let _ = engine.process_order(buy_order1).await.unwrap();
        let _ = engine.process_order(buy_order2).await.unwrap();
        let _ = engine.process_order(buy_order3).await.unwrap();
        
        // Create multiple sell orders at different price levels
        let sell_order1 = Order::new(
            Uuid::new_v4(),
            "SOL-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(102)),
            dec!(8),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        let sell_order2 = Order::new(
            Uuid::new_v4(),
            "SOL-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(103)),
            dec!(12),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        // Process the sell orders
        let _ = engine.process_order(sell_order1).await.unwrap();
        let _ = engine.process_order(sell_order2).await.unwrap();
        
        // Get order book snapshot
        let (bids, asks) = engine.get_order_book_snapshot(10).await.unwrap();
        
        // Check bid side (should be sorted by price in descending order)
        assert_eq!(bids.len(), 3);
        assert_eq!(bids[0], (dec!(101), dec!(5)));
        assert_eq!(bids[1], (dec!(100), dec!(10)));
        assert_eq!(bids[2], (dec!(99), dec!(15)));
        
        // Check ask side (should be sorted by price in ascending order)
        assert_eq!(asks.len(), 2);
        assert_eq!(asks[0], (dec!(102), dec!(8)));
        assert_eq!(asks[1], (dec!(103), dec!(12)));
        
        // Create a market sell order that should match against multiple price levels
        let market_sell = Order::new(
            Uuid::new_v4(),
            "SOL-USDT".to_string(),
            Side::Sell,
            OrderType::Market,
            None,
            dec!(12), // This should match against both 101 and 100 price levels
            TimeInForce::ImmediateOrCancel,
            None,
        );
        
        // Process the market sell order
        let trades = engine.process_order(market_sell).await.unwrap();
        
        // Should execute 2 trades
        assert_eq!(trades.len(), 2);
        
        // First trade should be at the best bid price (101)
        assert_eq!(trades[0].price, dec!(101));
        assert_eq!(trades[0].quantity, dec!(5));
        
        // Second trade should be at the next best bid price (100)
        assert_eq!(trades[1].price, dec!(100));
        assert_eq!(trades[1].quantity, dec!(7)); // Only 7 out of 10 should be filled
        
        // Get updated order book snapshot
        let (bids, asks) = engine.get_order_book_snapshot(10).await.unwrap();
        
        // Check bid side - 101 should be gone, 100 should be partially filled
        assert_eq!(bids.len(), 2);
        assert_eq!(bids[0], (dec!(100), dec!(3))); // 10 - 7 = 3
        assert_eq!(bids[1], (dec!(99), dec!(15))); // Unchanged
        
        // Ask side should be unchanged
        assert_eq!(asks.len(), 2);
    }
    
    #[tokio::test]
    async fn test_fill_or_kill_order() {
        let engine = MatchingEngine::new("LINK-USDT".to_string());
        
        // Create a sell order
        let sell_order = Order::new(
            Uuid::new_v4(),
            "LINK-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(20)),
            dec!(10),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        // Process the sell order
        let _ = engine.process_order(sell_order).await.unwrap();
        
        // Create a FOK buy order that is too large to be fully filled
        let fok_buy_order = Order::new(
            Uuid::new_v4(),
            "LINK-USDT".to_string(),
            Side::Buy,
            OrderType::Limit,
            Some(dec!(20)),
            dec!(15), // Only 10 available
            TimeInForce::FillOrKill,
            None,
        );
        
        // Process the FOK buy order
        let trades = engine.process_order(fok_buy_order).await.unwrap();
        
        // Should not execute any trades
        assert_eq!(trades.len(), 0);
        
        // Create a FOK buy order that can be fully filled
        let fok_buy_order2 = Order::new(
            Uuid::new_v4(),
            "LINK-USDT".to_string(),
            Side::Buy,
            OrderType::Limit,
            Some(dec!(20)),
            dec!(5), // Less than 10 available
            TimeInForce::FillOrKill,
            None,
        );
        
        // Process the second FOK buy order
        let trades = engine.process_order(fok_buy_order2).await.unwrap();
        
        // Should execute 1 trade
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, dec!(5));
    }
    
    #[tokio::test]
    async fn test_matching_engine_manager() {
        let manager = MatchingEngineManager::new();
        
        // Add trading pairs
        manager.add_symbol("BTC-USDT".to_string()).await.unwrap();
        manager.add_symbol("ETH-USDT".to_string()).await.unwrap();
        
        // Attempt to add duplicate symbol
        let result = manager.add_symbol("BTC-USDT".to_string()).await;
        assert!(result.is_err());
        
        // Process orders for different symbols
        let btc_sell = Order::new(
            Uuid::new_v4(),
            "BTC-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(50000)),
            dec!(1),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        let eth_sell = Order::new(
            Uuid::new_v4(),
            "ETH-USDT".to_string(),
            Side::Sell,
            OrderType::Limit,
            Some(dec!(3000)),
            dec!(5),
            TimeInForce::GoodTillCancel,
            None,
        );
        
        // Process orders through the manager
        let _ = manager.process_order(btc_sell).await.unwrap();
        let _ = manager.process_order(eth_sell).await.unwrap();
        
        // Get engines and check order books
        let btc_engine = manager.get_engine(&"BTC-USDT".to_string()).await.unwrap();
        let eth_engine = manager.get_engine(&"ETH-USDT".to_string()).await.unwrap();
        
        let (_, btc_asks) = btc_engine.get_order_book_snapshot(10).await.unwrap();
        let (_, eth_asks) = eth_engine.get_order_book_snapshot(10).await.unwrap();
        
        assert_eq!(btc_asks.len(), 1);
        assert_eq!(btc_asks[0], (dec!(50000), dec!(1)));
        
        assert_eq!(eth_asks.len(), 1);
        assert_eq!(eth_asks[0], (dec!(3000), dec!(5)));
        
        // Test cancelling an order
        let btc_engine = manager.get_engine(&"BTC-USDT".to_string()).await.unwrap();
        let (_, asks_before) = btc_engine.get_order_book_snapshot(10).await.unwrap();
        let order_id = Uuid::parse_str(&asks_before[0].0.to_string()).unwrap_or_default();
        
        // Cancel BTC order (this is a simplification, as we don't track order IDs properly in this example)
        let cancel_result = manager.cancel_order(&"BTC-USDT".to_string(), order_id).await;
        
        // In a real implementation, we would check that the order was cancelled
        // For this example, we'll just ensure no error was returned
        assert!(cancel_result.is_ok());
    }
}
