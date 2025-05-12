

use std::collections::HashMap;
use rust_decimal::Decimal;
use uuid::Uuid;
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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

/// Account model for balance tracking
#[derive(Debug, Clone)]
pub struct Account {
    pub user_id: UserId,
    pub balances: HashMap<String, Decimal>, // Asset symbol -> Balance
    pub available_balances: HashMap<String, Decimal>, // Asset symbol -> Available balance
}

impl Account {
    pub fn new(user_id: UserId) -> Self {
        Account {
            user_id,
            balances: HashMap::new(),
            available_balances: HashMap::new(),
        }
    }
    
    pub fn get_balance(&self, asset_symbol: &str) -> Decimal {
        *self.balances.get(asset_symbol).unwrap_or(&Decimal::ZERO)
    }
    
    pub fn get_available_balance(&self, asset_symbol: &str) -> Decimal {
        *self.available_balances.get(asset_symbol).unwrap_or(&Decimal::ZERO)
    }
    
    pub fn set_balance(&mut self, asset_symbol: &str, balance: Decimal, available_balance: Decimal) {
        self.balances.insert(asset_symbol.to_string(), balance);
        self.available_balances.insert(asset_symbol.to_string(), available_balance);
    }
}
