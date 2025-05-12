use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix::Addr;

use super::handlers::WebSocketConnection;

// Channel types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChannelType {
    OrderBook(String),  // Symbol
    Trades(String),     // Symbol
    Ticker(String),     // Symbol
    Kline(String, String), // Symbol, Interval
    UserOrders,
    UserTrades,
    UserWallet,
}

impl ChannelType {
    pub fn from_string(channel: &str) -> Option<Self> {
        let parts: Vec<&str> = channel.split(':').collect();
        
        match parts[0] {
            "orderbook" => {
                if parts.len() >= 2 {
                    Some(ChannelType::OrderBook(parts[1].to_string()))
                } else {
                    None
                }
            },
            "trades" => {
                if parts.len() >= 2 {
                    Some(ChannelType::Trades(parts[1].to_string()))
                } else {
                    None
                }
            },
            "ticker" => {
                if parts.len() >= 2 {
                    Some(ChannelType::Ticker(parts[1].to_string()))
                } else {
                    None
                }
            },
            "kline" => {
                if parts.len() >= 3 {
                    Some(ChannelType::Kline(parts[1].to_string(), parts[2].to_string()))
                } else {
                    None
                }
            },
            "user_orders" => Some(ChannelType::UserOrders),
            "user_trades" => Some(ChannelType::UserTrades),
            "user_wallet" => Some(ChannelType::UserWallet),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            ChannelType::OrderBook(symbol) => format!("orderbook:{}", symbol),
            ChannelType::Trades(symbol) => format!("trades:{}", symbol),
            ChannelType::Ticker(symbol) => format!("ticker:{}", symbol),
            ChannelType::Kline(symbol, interval) => format!("kline:{}:{}", symbol, interval),
            ChannelType::UserOrders => "user_orders".to_string(),
            ChannelType::UserTrades => "user_trades".to_string(),
            ChannelType::UserWallet => "user_wallet".to_string(),
        }
    }
}

// Manages channel subscriptions
pub struct ChannelManager {
    channels: Arc<Mutex<HashMap<ChannelType, Vec<Addr<WebSocketConnection>>>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn subscribe(&self, channel: ChannelType, addr: Addr<WebSocketConnection>) {
        let mut channels = self.channels.lock().unwrap();
        
        let subscribers = channels.entry(channel).or_insert_with(Vec::new);
        subscribers.push(addr);
    }
    
    pub fn unsubscribe(&self, channel: &ChannelType, addr: &Addr<WebSocketConnection>) {
        let mut channels = self.channels.lock().unwrap();
        
        if let Some(subscribers) = channels.get_mut(channel) {
            subscribers.retain(|subscriber| subscriber != addr);
            
            // Remove channel if no subscribers left
            if subscribers.is_empty() {
                channels.remove(channel);
            }
        }
    }
    
    pub fn broadcast(&self, channel: &ChannelType, message: &str) {
        let channels = self.channels.lock().unwrap();
        
        if let Some(subscribers) = channels.get(channel) {
            for subscriber in subscribers {
                let message = message.to_string();
                subscriber.do_send(WebSocketConnection::Message(message));
            }
        }
    }
}

impl Clone for ChannelManager {
    fn clone(&self) -> Self {
        Self {
            channels: Arc::clone(&self.channels),
        }
    }
}
