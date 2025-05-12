mod channels;
mod handlers;

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Duration, Instant};

use channels::ChannelManager;
use handlers::WebSocketSession;

// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

// WebSocket connection handler
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<crate::config::Config>,
) -> Result<HttpResponse, Error> {
    // Extract token from query parameters
    let query_params = req.query_string();
    let token = query_params
        .split('&')
        .find_map(|param| {
            if param.starts_with("token=") {
                Some(param[6..].to_string())
            } else {
                None
            }
        });

    // Create WebSocket session
    let session = WebSocketSession {
        id: uuid::Uuid::new_v4(),
        heartbeat: Instant::now(),
        channel_subscriptions: Vec::new(),
        channel_manager: ChannelManager::new(),
        token,
        config: config.get_ref().clone(),
    };

    // Start WebSocket connection
    let resp = ws::start(session, &req, stream)?;

    Ok(resp)
}
