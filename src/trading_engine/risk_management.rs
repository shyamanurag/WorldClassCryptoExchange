// src/trading_engine/risk_manager.rs

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use parking_lot::RwLock;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};
use uuid::Uuid;

use crate::models::{
    Side, OrderId, Symbol, Price, Quantity, Order, OrderStatus, 
    TimeInForce, UserId, Trade, Position
};

/// Represents the risk check result
#[derive(Debug, Clone)]
pub enum RiskCheckResult {
    /// Order is accepted
    Accepted,
    /// Order is rejected with a reason
    Rejected { reason: String },
    /// Order is modified to comply with risk limits
    Modified { 
        original_order: Arc<RwLock<Order>>,
        modified_order: Arc<RwLock<Order>>,
        reason: String,
    },
}

/// Tracks position and risk limits for a user
#[derive(Debug, Clone)]
pub struct UserRiskProfile {
    /// User ID
    pub user_id: UserId,
    /// Current positions by symbol
    pub positions: HashMap<Symbol, Position>,
    /// Maximum position size allowed
    pub max_position_size: HashMap<Symbol, Decimal>,
    /// Maximum notional value allowed
    pub max_notional_value: Decimal,
    /// Current order count
    pub order_count: usize,
    /// Maximum order count allowed
    pub max_order_count: usize,
    /// Rate limit: orders per minute
    pub orders_per_minute: usize,
    /// Order count in the current minute
    pub current_minute_orders: usize,
    /// Last minute timestamp
    pub last_minute: u64,
    /// Is the user allowed to trade
    pub trading_enabled: bool,
}

impl UserRiskProfile {
    /// Create a new user risk profile with default limits
    pub fn new(user_id: UserId) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            user_id,
            positions: HashMap::new(),
            max_position_size: HashMap::new(),
            max_notional_value: dec!(1_000_000),
            order_count: 0,
            max_order_count: 1000,
            orders_per_minute: 300,
            current_minute_orders: 0,
            last_minute: now / 60,
            trading_enabled: true,
        }
    }
    
    /// Set maximum position size for a symbol
    pub fn set_max_position_size(&mut self, symbol: Symbol, size: Decimal) {
        self.max_position_size.insert(symbol, size);
    }
    
    /// Update position based on a trade
    pub fn update_position(&mut self, symbol: &Symbol, side: Side, quantity: Decimal, price: Decimal) {
        let position = self.positions.entry(symbol.clone()).or_insert_with(|| {
            Position {
                symbol: symbol.clone(),
                quantity: Decimal::ZERO,
                average_price: Decimal::ZERO,
                unrealized_pnl: Decimal::ZERO,
                realized_pnl: Decimal::ZERO,
                last_update_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64,
            }
        });

        // Update position based on trade side
        match side {
            Side::Buy => {
                // Update average price using weighted average
                if position.quantity + quantity > Decimal::ZERO {
                    position.average_price = 
                        (position.quantity * position.average_price + quantity * price) / 
                        (position.quantity + quantity);
                }
                position.quantity += quantity;
            },
            Side::Sell => {
                // Calculate P&L for partial position close
                if position.quantity > Decimal::ZERO {
                    let realized_pnl = quantity * (price - position.average_price);
                    position.realized_pnl += realized_pnl;
                }
                position.quantity -= quantity;
                
                // Reset average price if position flipped
                if position.quantity < Decimal::ZERO {
                    position.average_price = price;
                }
            },
        }
        
        position.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
    }
    
    /// Check rate limit
    pub fn check_rate_limit(&mut self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let current_minute = now / 60;
        
        if current_minute != self.last_minute {
            // Reset counter for new minute
            self.current_minute_orders = 0;
            self.last_minute = current_minute;
        }
        
        self.current_minute_orders += 1;
        
        self.current_minute_orders <= self.orders_per_minute
    }
    
    /// Get total notional value of all positions
    pub fn get_total_notional_value(&self, current_prices: &HashMap<Symbol, Decimal>) -> Decimal {
        self.positions.iter()
            .map(|(symbol, position)| {
                if let Some(&price) = current_prices.get(symbol) {
                    position.quantity.abs() * price
                } else {
                    position.quantity.abs() * position.average_price
                }
            })
            .sum()
    }
}

/// Configuration for the risk manager
#[derive(Debug, Clone)]
pub struct RiskManagerConfig {
    /// Default max position size as a percentage of account equity
    pub default_max_position_size_pct: Decimal,
    /// Default max notional value
    pub default_max_notional_value: Decimal,
    /// Default max order count
    pub default_max_order_count: usize,
    /// Default orders per minute
    pub default_orders_per_minute: usize,
    /// Circuit breaker percentage
    pub circuit_breaker_pct: Decimal,
    /// Price bands for circuit breakers by symbol
    pub price_bands: HashMap<Symbol, (Decimal, Decimal)>,
    /// Minimum order size by symbol
    pub min_order_size: HashMap<Symbol, Decimal>,
    /// Maximum order size by symbol
    pub max_order_size: HashMap<Symbol, Decimal>,
    /// Tick size by symbol
    pub tick_size: HashMap<Symbol, Decimal>,
}

impl Default for RiskManagerConfig {
    fn default() -> Self {
        let mut price_bands = HashMap::new();
        price_bands.insert("BTC/USD".to_string(), (dec!(0.95), dec!(1.05))); // 5% band
        price_bands.insert("ETH/USD".to_string(), (dec!(0.93), dec!(1.07))); // 7% band
        
        let mut min_order_size = HashMap::new();
        min_order_size.insert("BTC/USD".to_string(), dec!(0.001));
        min_order_size.insert("ETH/USD".to_string(), dec!(0.01));
        
        let mut max_order_size = HashMap::new();
        max_order_size.insert("BTC/USD".to_string(), dec!(10));
        max_order_size.insert("ETH/USD".to_string(), dec!(100));
        
        let mut tick_size = HashMap::new();
        tick_size.insert("BTC/USD".to_string(), dec!(0.5));
        tick_size.insert("ETH/USD".to_string(), dec!(0.1));

        Self {
            default_max_position_size_pct: dec!(0.2), // 20% of equity
            default_max_notional_value: dec!(1_000_000),
            default_max_order_count: 1000,
            default_orders_per_minute: 300,
            circuit_breaker_pct: dec!(0.1), // 10% price move
            price_bands,
            min_order_size,
            max_order_size,
            tick_size,
        }
    }
}

/// The risk manager that checks orders for compliance with risk limits
pub struct RiskManager {
    /// User risk profiles
    user_profiles: HashMap<UserId, RwLock<UserRiskProfile>>,
    /// Current market prices
    market_prices: RwLock<HashMap<Symbol, Decimal>>,
    /// Last trade prices by symbol
    last_trade_prices: RwLock<HashMap<Symbol, Decimal>>,
    /// Order ID to user ID mapping
    order_to_user: HashMap<OrderId, UserId>,
    /// Circuit breaker status by symbol
    circuit_breakers: RwLock<HashMap<Symbol, bool>>,
    /// Configuration for the risk manager
    config: RiskManagerConfig,
}

impl RiskManager {
    /// Create a new risk manager with the specified configuration
    pub fn new(config: RiskManagerConfig) -> Self {
        Self {
            user_profiles: HashMap::new(),
            market_prices: RwLock::new(HashMap::new()),
            last_trade_prices: RwLock::new(HashMap::new()),
            order_to_user: HashMap::new(),
            circuit_breakers: RwLock::new(HashMap::new()),
            config,
        }
    }
    
    /// Register a user with the risk manager
    pub fn register_user(&mut self, user_id: UserId) {
        if !self.user_profiles.contains_key(&user_id) {
            let profile = UserRiskProfile::new(user_id);
            self.user_profiles.insert(user_id, RwLock::new(profile));
        }
    }
    
    /// Check if a user is registered
    pub fn is_user_registered(&self, user_id: &UserId) -> bool {
        self.user_profiles.contains_key(user_id)
    }
    
    /// Get a user's risk profile
    pub fn get_user_profile(&self, user_id: &UserId) -> Option<&RwLock<UserRiskProfile>> {
        self.user_profiles.get(user_id)
    }
    
    /// Set maximum position size for a user and symbol
    pub fn set_user_max_position_size(&self, user_id: &UserId, symbol: Symbol, size: Decimal) -> Result<()> {
        if let Some(profile_lock) = self.user_profiles.get(user_id) {
            let mut profile = profile_lock.write();
            profile.set_max_position_size(symbol, size);
            Ok(())
        } else {
            Err(anyhow!("User not registered"))
        }
    }
    
    /// Set trading enabled status for a user
    pub fn set_trading_enabled(&self, user_id: &UserId, enabled: bool) -> Result<()> {
        if let Some(profile_lock) = self.user_profiles.get(user_id) {
            let mut profile = profile_lock.write();
            profile.trading_enabled = enabled;
            Ok(())
        } else {
            Err(anyhow!("User not registered"))
        }
    }
    
    /// Update market price for a symbol
    pub fn update_market_price(&self, symbol: Symbol, price: Decimal) {
        let mut prices = self.market_prices.write();
        prices.insert(symbol, price);
    }
    
    /// Process a trade and update risk profiles
    pub fn process_trade(&self, trade: &Trade) -> Result<()> {
        // Update last trade price
        {
            let mut last_prices = self.last_trade_prices.write();
            last_prices.insert(trade.symbol.clone(), trade.price);
        }
        
        // Check for circuit breakers
        self.check_circuit_breakers(&trade.symbol, trade.price)?;
        
        // Update user positions
        if let Some(user_id) = self.get_user_for_order(&trade.taker_order_id) {
            if let Some(profile_lock) = self.user_profiles.get(&user_id) {
                let mut profile = profile_lock.write();
                profile.update_position(&trade.symbol, trade.side, trade.quantity, trade.price);
            }
        }
        
        // For the maker order, we need to update the opposite side
        if let Some(user_id) = self.get_user_for_order(&trade.maker_order_id) {
            if let Some(profile_lock) = self.user_profiles.get(&user_id) {
                let mut profile = profile_lock.write();
                // Maker's side is opposite of the trade's reported side
                let maker_side = match trade.side {
                    Side::Buy => Side::Sell,
                    Side::Sell => Side::Buy,
                };
                profile.update_position(&trade.symbol, maker_side, trade.quantity, trade.price);
            }
        }
        
        Ok(())
    }
    
    /// Check circuit breakers for a symbol
    fn check_circuit_breakers(&self, symbol: &Symbol, current_price: Decimal) -> Result<()> {
        let mut breakers = self.circuit_breakers.write();
        
        // Get previous price to compare
        let last_prices = self.last_trade_prices.read();
        
        if let Some(&last_price) = last_prices.get(symbol) {
            if last_price > Decimal::ZERO {
                // Calculate price move percentage
                let price_move = (current_price - last_price).abs() / last_price;
                
                // Check if price move exceeds the circuit breaker threshold
                if price_move > self.config.circuit_breaker_pct {
                    info!("Circuit breaker triggered for {}: price move {}%", 
                         symbol, price_move * Decimal::from(100));
                    breakers.insert(symbol.clone(), true);
                    return Ok(());
                }
            }
        }
        
        // Check if price is within allowed bands
        if let Some(&(lower, upper)) = self.config.price_bands.get(symbol) {
            let market_prices = self.market_prices.read();
            
            if let Some(&reference_price) = market_prices.get(symbol) {
                if reference_price > Decimal::ZERO {
                    let lower_bound = reference_price * lower;
                    let upper_bound = reference_price * upper;
                    
                    if current_price < lower_bound || current_price > upper_bound {
                        info!("Circuit breaker triggered for {}: price {} outside of bands ({}, {})", 
                             symbol, current_price, lower_bound, upper_bound);
                        breakers.insert(symbol.clone(), true);
                        return Ok(());
                    }
                }
            }
        }
        
        // If we get here, no circuit breaker was triggered
        breakers.insert(symbol.clone(), false);
        Ok(())
    }
    
    /// Validate an order against risk limits
    pub fn validate_order(&self, order: Arc<RwLock<Order>>, user_id: &UserId) -> Result<RiskCheckResult> {
        // Check if symbol has an active circuit breaker
        {
            let breakers = self.circuit_breakers.read();
            let order_ref = order.read();
            if let Some(&active) = breakers.get(&order_ref.symbol) {
                if active {
                    return Ok(RiskCheckResult::Rejected { 
                        reason: format!("Circuit breaker active for {}", order_ref.symbol) 
                    });
                }
            }
        }
        
        // Check if user is registered
        if !self.is_user_registered(user_id) {
            return Ok(RiskCheckResult::Rejected { 
                reason: "User not registered with risk manager".to_string() 
            });
        }
        
        // Get user profile
        let profile_lock = self.user_profiles.get(user_id).unwrap();
        let mut profile = profile_lock.write();
        
        // Check if trading is enabled for user
        if !profile.trading_enabled {
            return Ok(RiskCheckResult::Rejected { 
                reason: "Trading disabled for user".to_string() 
            });
        }
        
        // Check rate limit
        if !profile.check_rate_limit() {
            return Ok(RiskCheckResult::Rejected { 
                reason: "Rate limit exceeded".to_string() 
            });
        }
        
        // Check order count
        if profile.order_count >= profile.max_order_count {
            return Ok(RiskCheckResult::Rejected { 
                reason: "Maximum order count exceeded".to_string() 
            });
        }
        
        let order_ref = order.read();
        
        // Check minimum and maximum order size
        if let Some(&min_size) = self.config.min_order_size.get(&order_ref.symbol) {
            if order_ref.quantity < min_size {
                return Ok(RiskCheckResult::Rejected { 
                    reason: format!("Order size {} below minimum {}", order_ref.quantity, min_size) 
                });
            }
        }
        
        if let Some(&max_size) = self.config.max_order_size.get(&order_ref.symbol) {
            if order_ref.quantity > max_size {
                return Ok(RiskCheckResult::Rejected { 
                    reason: format!("Order size {} above maximum {}", order_ref.quantity, max_size) 
                });
            }
        }
        
        // Check tick size for limit orders
        if let Some(price) = order_ref.price {
            if let Some(&tick_size) = self.config.tick_size.get(&order_ref.symbol) {
                if price % tick_size != Decimal::ZERO {
                    return Ok(RiskCheckResult::Rejected { 
                        reason: format!("Price {} not a multiple of tick size {}", price, tick_size) 
                    });
                }
            }
        }
        
        // Check position limits
        if let Some(&max_position) = profile.max_position_size.get(&order_ref.symbol) {
            let position = profile.positions
                .get(&order_ref.symbol)
                .map(|p| p.quantity)
                .unwrap_or(Decimal::ZERO);
            
            let new_position = match order_ref.side {
                Side::Buy => position + order_ref.quantity,
                Side::Sell => position - order_ref.quantity,
            };
            
            if new_position.abs() > max_position {
                return Ok(RiskCheckResult::Rejected { 
                    reason: format!("Position size {} would exceed limit {}", new_position.abs(), max_position) 
                });
            }
        }
        
        // Check notional value
        let market_prices = self.market_prices.read();
        let total_notional = profile.get_total_notional_value(&market_prices);
        
        let order_notional = if let Some(&price) = market_prices.get(&order_ref.symbol) {
            order_ref.quantity * price
        } else if let Some(order_price) = order_ref.price {
            order_ref.quantity * order_price
        } else {
            // If no price available, reject market orders
            return Ok(RiskCheckResult::Rejected { 
                reason: "Cannot determine order notional value".to_string() 
            });
        };
        
        if total_notional + order_notional > profile.max_notional_value {
            return Ok(RiskCheckResult::Rejected { 
                reason: format!("Total notional value {} would exceed limit {}", 
                                total_notional + order_notional, profile.max_notional_value) 
            });
        }
        
        // If we get here, all risk checks have passed
        profile.order_count += 1;
        
        // Store order-to-user mapping
        self.order_to_user.insert(order_ref.id, *user_id);
        
        Ok(RiskCheckResult::Accepted)
    }
    
    /// Get the user ID associated with an order
    fn get_user_for_order(&self, order_id: &OrderId) -> Option<UserId> {
        self.order_to_user.get(order_id).copied()
    }
    
    /// Reset circuit breakers for all symbols
    pub fn reset_circuit_breakers(&self) {
        let mut breakers = self.circuit_breakers.write();
        for (_, value) in breakers.iter_mut() {
            *value = false;
        }
    }
    
    /// Reset circuit breaker for a specific symbol
    pub fn reset_circuit_breaker(&self, symbol: &Symbol) {
        let mut breakers = self.circuit_breakers.write();
        if let Some(value) = breakers.get_mut(symbol) {
            *value = false;
        }
    }
    
    /// Check if a circuit breaker is active for a symbol
    pub fn is_circuit_breaker_active(&self, symbol: &Symbol) -> bool {
        let breakers = self.circuit_breakers.read();
        breakers.get(symbol).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    fn create_order(id: &str, symbol: &str, side: Side, price: Option<Decimal>, quantity: Decimal) -> Arc<RwLock<Order>> {
        let order = Order {
            id: Uuid::parse_str(id).unwrap(),
            symbol: symbol.to_string(),
            side,
            order_type: if price.is_some() { "LIMIT".to_string() } else { "MARKET".to_string() },
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::New,
            time_in_force: TimeInForce::GoodTillCancel,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        };
        
        Arc::new(RwLock::new(order))
    }
    
    #[test]
    fn test_register_user() {
        let config = RiskManagerConfig::default();
        let mut risk_manager = RiskManager::new(config);
        
        let user_id = Uuid::new_v4();
        risk_manager.register_user(user_id);
        
        assert!(risk_manager.is_user_registered(&user_id));
    }
    
    #[test]
    fn test_validate_order_successful() {
        let config = RiskManagerConfig::default();
        let mut risk_manager = RiskManager::new(config);
        
        let user_id = Uuid::new_v4();
        risk_manager.register_user(user_id);
        
        // Set market price
        risk_manager.update_market_price("BTC/USD".to_string(), dec!(50000));
        
        // Create valid order
        let order = create_order(
            "00000000-0000-0000-0000-000000000001",
            "BTC/USD",
            Side::Buy,
            Some(dec!(50000)),
            dec!(0.1)
        );
        
        let result = risk_manager.validate_order(order, &user_id).unwrap();
        
        match result {
            RiskCheckResult::Accepted => {
                // Expected result
            },
            _ => panic!("Expected order to be accepted"),
        }
    }
    
    #[test]
    fn test_validate_order_size_too_small() {
        let config = RiskManagerConfig::default();
        let mut risk_manager = RiskManager::new(config);
        
        let user_id = Uuid::new_v4();
        risk_manager.register_user(user_id);
        
        // Set market price
        risk_manager.update_market_price("BTC/USD".to_string(), dec!(50000));
        
        // Create order with size that's too small
        let order = create_order(
            "00000000-0000-0000-0000-000000000001",
            "BTC/USD",
            Side::Buy,
            Some(dec!(50000)),
            dec!(0.0001) // Below minimum of 0.001
        );
        
        let result = risk_manager.validate_order(order, &user_id).unwrap();
        
        match result {
            RiskCheckResult::Rejected { reason } => {
                assert!(reason.contains("below minimum"));
            },
            _ => panic!("Expected order to be rejected"),
        }
    }
    
    #[test]
    fn test_position_updates() {
        let config = RiskManagerConfig::default();
        let mut risk_manager = RiskManager::new(config);
        
        let user_id = Uuid::new_v4();
        risk_manager.register_user(user_id);
        
        // Create a trade where the user is the taker
        let trade = Trade::new(
            "BTC/USD".to_string(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            dec!(50000),
            dec!(0.1),
            Side::Buy
        );
        
        // Associate the taker order with our user
        risk_manager.order_to_user.insert(trade.taker_order_id, user_id);
        
        // Process the trade
        risk_manager.process_trade(&trade).unwrap();
        
        // Check position was updated correctly
        let profile = risk_manager.get_user_profile(&user_id).unwrap().read();
        let position = profile.positions.get("BTC/USD").unwrap();
        
        assert_eq!(position.quantity, dec!(0.1));
        assert_eq!(position.average_price, dec!(50000));
    }
    
    #[test]
    fn test_circuit_breaker() {
        let mut config = RiskManagerConfig::default();
        config.circuit_breaker_pct = dec!(0.05); // 5% move
        
        let risk_manager = RiskManager::new(config);
        
        // Initialize with a last trade price
        {
            let mut last_prices = risk_manager.last_trade_prices.write();
            last_prices.insert("BTC/USD".to_string(), dec!(50000));
        }
        
        // Set reference price
        risk_manager.update_market_price("BTC/USD".to_string(), dec!(50000));
        
        // Test with price in range
        risk_manager.check_circuit_breakers(
            &"BTC/USD".to_string(), 
            dec!(51000)  // 2% move
        ).unwrap();
        
        assert!(!risk_manager.is_circuit_breaker_active(&"BTC/USD".to_string()));
        
        // Test with price that triggers circuit breaker
        risk_manager.check_circuit_breakers(
            &"BTC/USD".to_string(), 
            dec!(53000)  // 6% move
        ).unwrap();
        
        assert!(risk_manager.is_circuit_breaker_active(&"BTC/USD".to_string()));
        
        // Reset circuit breaker
        risk_manager.reset_circuit_breaker(&"BTC/USD".to_string());
        
        assert!(!risk_manager.is_circuit_breaker_active(&"BTC/USD".to_string()));
    }
}
