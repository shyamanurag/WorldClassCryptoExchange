// src/db/models.rs - Database data models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Decimal;
use uuid::Uuid;
use std::fmt;
use std::str::FromStr;

/// Order side (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_side", rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "buy"),
            OrderSide::Sell => write!(f, "sell"),
        }
    }
}

impl FromStr for OrderSide {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buy" => Ok(OrderSide::Buy),
            "sell" => Ok(OrderSide::Sell),
            _ => Err(format!("Invalid order side: {}", s)),
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_type", rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    OCO, // One-Cancels-Other order
    Iceberg, // Order that automatically posts small portions of total order quantity
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderType::Market => write!(f, "market"),
            OrderType::Limit => write!(f, "limit"),
            OrderType::StopLoss => write!(f, "stop_loss"),
            OrderType::StopLossLimit => write!(f, "stop_loss_limit"),
            OrderType::TakeProfit => write!(f, "take_profit"),
            OrderType::TakeProfitLimit => write!(f, "take_profit_limit"),
            OrderType::OCO => write!(f, "oco"),
            OrderType::Iceberg => write!(f, "iceberg"),
        }
    }
}

impl FromStr for OrderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "market" => Ok(OrderType::Market),
            "limit" => Ok(OrderType::Limit),
            "stop_loss" => Ok(OrderType::StopLoss),
            "stop_loss_limit" => Ok(OrderType::StopLossLimit),
            "take_profit" => Ok(OrderType::TakeProfit),
            "take_profit_limit" => Ok(OrderType::TakeProfitLimit),
            "oco" => Ok(OrderType::OCO),
            "iceberg" => Ok(OrderType::Iceberg),
            _ => Err(format!("Invalid order type: {}", s)),
        }
    }
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderStatus::New => write!(f, "new"),
            OrderStatus::PartiallyFilled => write!(f, "partially_filled"),
            OrderStatus::Filled => write!(f, "filled"),
            OrderStatus::Canceled => write!(f, "canceled"),
            OrderStatus::Rejected => write!(f, "rejected"),
            OrderStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Time in force options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "time_in_force", rename_all = "lowercase")]
pub enum TimeInForce {
    GoodTillCancel, // Order valid until canceled
    ImmediateOrCancel, // Order must be executed immediately, if not - cancel
    FillOrKill, // Order must be executed with the entire quantity immediately, if not - cancel
    GoodTillDate, // Order valid until a specified date/time
}

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeInForce::GoodTillCancel => write!(f, "good_till_cancel"),
            TimeInForce::ImmediateOrCancel => write!(f, "immediate_or_cancel"),
            TimeInForce::FillOrKill => write!(f, "fill_or_kill"),
            TimeInForce::GoodTillDate => write!(f, "good_till_date"),
        }
    }
}

impl FromStr for TimeInForce {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "good_till_cancel" | "gtc" => Ok(TimeInForce::GoodTillCancel),
            "immediate_or_cancel" | "ioc" => Ok(TimeInForce::ImmediateOrCancel),
            "fill_or_kill" | "fok" => Ok(TimeInForce::FillOrKill),
            "good_till_date" | "gtd" => Ok(TimeInForce::GoodTillDate),
            _ => Err(format!("Invalid time in force: {}", s)),
        }
    }
}

/// Order model representing a trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub visible_quantity: Option<Decimal>, // For iceberg orders
    pub expiry_time: Option<DateTime<Utc>>, // For GoodTillDate orders
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    /// Create a new order
    pub fn new(
        user_id: Uuid,
        symbol: String,
        side: OrderSide,
        order_type: OrderType,
        price: Option<Decimal>,
        stop_price: Option<Decimal>,
        quantity: Decimal,
        time_in_force: TimeInForce,
        visible_quantity: Option<Decimal>,
        expiry_time: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            user_id,
            symbol,
            side,
            order_type,
            price,
            stop_price,
            quantity,
            filled_quantity: Decimal::from(0),
            status: OrderStatus::New,
            time_in_force,
            visible_quantity,
            expiry_time,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if the order is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderStatus::New | OrderStatus::PartiallyFilled)
    }
    
    /// Calculate the remaining quantity to be filled
    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }
    
    /// Check if the order is fillable at a specific price
    pub fn is_fillable_at(&self, market_price: Decimal) -> bool {
        // Market orders are always fillable
        if self.order_type == OrderType::Market {
            return true;
        }
        
        // For limit orders, check if the price is good enough
        match (self.side, self.price) {
            (OrderSide::Buy, Some(price)) => market_price <= price,
            (OrderSide::Sell, Some(price)) => market_price >= price,
            _ => false,
        }
    }
    
    /// Fill a portion of this order
    pub fn fill(&mut self, fill_quantity: Decimal, fill_price: Decimal) -> Result<Trade, String> {
        // Check if order can be filled
        if !self.is_active() {
            return Err(format!("Order {} is not active", self.id));
        }
        
        // Check if there's enough quantity left to fill
        let remaining = self.remaining_quantity();
        if fill_quantity > remaining {
            return Err(format!(
                "Fill quantity {} exceeds remaining quantity {}",
                fill_quantity, remaining
            ));
        }
        
        // Update filled quantity
        self.filled_quantity += fill_quantity;
        
        // Update status
        if self.filled_quantity >= self.quantity {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
        
        // Update timestamp
        self.updated_at = Utc::now();
        
        // Create a trade record
        let trade = Trade {
            id: Uuid::new_v4(),
            symbol: self.symbol.clone(),
            price: fill_price,
            quantity: fill_quantity,
            side: self.side,
            order_id: self.id,
            user_id: self.user_id,
            executed_at: Utc::now(),
            fee: Decimal::from(0), // Fee should be calculated elsewhere
        };
        
        Ok(trade)
    }
}

/// Trade model representing a completed trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub executed_at: DateTime<Utc>,
    pub fee: Decimal,
}

/// Match model representing two orders that were matched together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: Uuid,
    pub symbol: String,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub executed_at: DateTime<Utc>,
}

/// User model representing a registered user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset model representing a cryptocurrency or fiat currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub asset_type: AssetType,
    pub decimals: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "asset_type", rename_all = "lowercase")]
pub enum AssetType {
    Cryptocurrency,
    Fiat,
    Token,
}

/// Trading pair model representing a pair of assets that can be traded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub id: Uuid,
    pub symbol: String,
    pub base_asset_id: Uuid,
    pub quote_asset_id: Uuid,
    pub price_precision: i32,
    pub quantity_precision: i32,
    pub min_order_size: Decimal,
    pub max_order_size: Option<Decimal>,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Wallet model representing a user's wallet for a specific asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub balance: Decimal,
    pub available_balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account model representing a user's account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_type: AccountType,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "account_type", rename_all = "lowercase")]
pub enum AccountType {
    Spot,
    Margin,
    Futures,
}

/// Deposit model representing a deposit transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub amount: Decimal,
    pub address: Option<String>,
    pub txid: Option<String>,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Withdrawal model representing a withdrawal transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub amount: Decimal,
    pub fee: Decimal,
    pub address: String,
    pub txid: Option<String>,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Canceled,
}

/// Order book entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// Order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub timestamp: DateTime<Utc>,
}

/// Ticker data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub last_price: Decimal,
    pub bid_price: Decimal,
    pub ask_price: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub volume_24h: Decimal,
    pub price_change_24h: Decimal,
    pub price_change_percent_24h: f64,
    pub timestamp: DateTime<Utc>,
