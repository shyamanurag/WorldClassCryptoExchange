use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

use crate::config::Config;
use super::channels::{ChannelManager, ChannelType};
use super::{HEARTBEAT_INTERVAL, CLIENT_TIMEOUT};

// WebSocket messages
#[derive(Message)]
#[rtype(result = "()")]
pub enum WebSocketConnection {
    Message(String),
}

// WebSocket session
pub struct WebSocketSession {
    pub id: Uuid,
    pub heartbeat: Instant,
    pub channel_subscriptions: Vec<ChannelType>,
    pub channel_manager: ChannelManager,
    pub token: Option<String>,
    pub config: Config,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WebSocketMessage {
    Subscribe {
        channel: String,
    },
    Unsubscribe {
        channel: String,
    },
    Ping {},
    Ping2 {},
}

// WebSocket response types
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketResponse {
    Subscribed {
        channel: String,
    },
    Unsubscribed {
        channel: String,
    },
    Error {
        code: u16,
        message: String,
    },
    Pong {},
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    // Start heartbeat process on session start
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        
        info!("WebSocket connection established: {}", self.id);
    }

    // Clean up on session end
    fn stopped(&mut self, _: &mut Self::Context) {
        info!("WebSocket connection closed: {}", self.id);
        
        // Unsubscribe from all channels
        for channel in &self.channel_subscriptions {
            self.channel_manager.unsubscribe(channel, &ctx.address());
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                debug!("Received WebSocket message: {}", text);
                
                // Parse message
                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(message) => match message {
                        WebSocketMessage::Subscribe { channel } => {
                            self.handle_subscribe(ctx, &channel);
                        }
                        WebSocketMessage::Unsubscribe { channel } => {
                            self.handle_unsubscribe(ctx, &channel);
                        }
                        WebSocketMessage::Ping {} | WebSocketMessage::Ping2 {} => {
                            self.heartbeat = Instant::now();
                            let response = WebSocketResponse::Pong {};
                            ctx.text(serde_json::to_string(&response).unwrap());
                        }
                    },
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                        let response = WebSocketResponse::Error {
                            code: 400,
                            message: "Invalid message format".to_string(),
                        };
                        ctx.text(serde_json::to_string(&response).unwrap());
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                warn!("Unexpected binary message: {:?}", bin);
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket connection closing: {:?}", reason);
                ctx.close(reason);
            }
            _ => {}
        }
    }
}

impl WebSocketSession {
    // Send heartbeat ping
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check client heartbeat
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                warn!("WebSocket client timeout: {}", act.id);
                ctx.stop();
                return;
            }
            
            ctx.ping(b"");
        });
    }

    // Handle subscribe message
    fn handle_subscribe(&mut self, ctx: &mut ws::WebsocketContext<Self>, channel: &str) {
        if let Some(channel_type) = ChannelType::from_string(channel) {
            // Check if user-specific channel requires authentication
            match &channel_type {
                ChannelType::UserOrders | ChannelType::UserTrades | ChannelType::UserWallet => {
                    if self.token.is_none() {
                        let response = WebSocketResponse::Error {
                            code: 401,
                            message: "Authentication required for user channels".to_string(),
                        };
                        ctx.text(serde_json::to_string(&response).unwrap());
                        return;
                    }
                    
                    // Validate token
                    // In a real implementation, you would verify the token and extract user ID
                    // For now, we'll assume the token is valid
                }
                _ => {}
            }
            
            // Subscribe to channel
            self.channel_manager.subscribe(channel_type.clone(), ctx.address());
            self.channel_subscriptions.push(channel_type.clone());
            
            // Send subscription confirmation
            let response = WebSocketResponse::Subscribed {
                channel: channel_type.to_string(),
            };
            ctx.text(serde_json::to_string(&response).unwrap());
            
            info!("Subscribed to channel: {}", channel);
        } else {
            let response = WebSocketResponse::Error {
                code: 400,
                message: format!("Invalid channel: {}", channel),
            };
            ctx.text(serde_json::to_string(&response).unwrap());
        }
    }

    // Handle unsubscribe message
    fn handle_unsubscribe(&mut self, ctx: &mut ws::WebsocketContext<Self>, channel: &str) {
        if let Some(channel_type) = ChannelType::from_string(channel) {
            // Unsubscribe from channel
            self.channel_manager.unsubscribe(&channel_type, &ctx.address());
            self.channel_subscriptions.retain(|c| c != &channel_type);
            
            // Send unsubscription confirmation
            let response = WebSocketResponse::Unsubscribed {
                channel: channel_type.to_string(),
            };
            ctx.text(serde_json::to_string(&response).unwrap());
            
            info!("Unsubscribed from channel: {}", channel);
        } else {
            let response = WebSocketResponse::Error {
                code: 400,
                message: format!("Invalid channel: {}", channel),
            };
            ctx.text(serde_json::to_string(&response).unwrap());
        }
    }
}

// Handler for WebSocketConnection messages
impl Handler<WebSocketConnection> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: WebSocketConnection, ctx: &mut Self::Context) {
        match msg {
            WebSocketConnection::Message(message) => {
                ctx.text(message);
            }
        }
    }
}
