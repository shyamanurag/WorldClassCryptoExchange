use actix_web::web;
use super::handlers::{user, market, order, wallet};

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    // API version prefix - all routes will be under /api/v1
    cfg.service(
        web::scope("/api/v1")
            // User routes
            .service(
                web::scope("/users")
                    .route("", web::post().to(user::create_user))
                    .route("", web::get().to(user::get_users))
                    .route("/{id}", web::get().to(user::get_user))
                    .route("/{id}", web::put().to(user::update_user))
                    .route("/{id}", web::delete().to(user::delete_user))
            )
            // Authentication routes
            .service(
                web::scope("/auth")
                    .route("/login", web::post().to(user::login))
                    .route("/logout", web::post().to(user::logout))
                    .route("/refresh", web::post().to(user::refresh_token))
                    .route("/me", web::get().to(user::get_current_user))
                    .route("/2fa/enable", web::post().to(user::enable_2fa))
                    .route("/2fa/verify", web::post().to(user::verify_2fa))
                    .route("/2fa/disable", web::post().to(user::disable_2fa))
            )
            // Market data routes
            .service(
                web::scope("/markets")
                    .route("", web::get().to(market::get_markets))
                    .route("/{symbol}", web::get().to(market::get_market_details))
                    .route("/{symbol}/ticker", web::get().to(market::get_ticker))
                    .route("/{symbol}/orderbook", web::get().to(market::get_orderbook))
                    .route("/{symbol}/trades", web::get().to(market::get_trades))
                    .route("/{symbol}/candles", web::get().to(market::get_candles))
            )
            // Order routes
            .service(
                web::scope("/orders")
                    .route("", web::post().to(order::create_order))
                    .route("", web::get().to(order::get_orders))
                    .route("/{id}", web::get().to(order::get_order))
                    .route("/{id}", web::delete().to(order::cancel_order))
                    .route("/history", web::get().to(order::get_order_history))
            )
            // Wallet routes
            .service(
                web::scope("/wallet")
                    .route("/balances", web::get().to(wallet::get_balances))
                    .route("/transactions", web::get().to(wallet::get_transactions))
                    .route("/deposit-address/{currency}", web::get().to(wallet::get_deposit_address))
                    .route("/withdraw", web::post().to(wallet::create_withdrawal))
                    .route("/estimate-withdrawal-fee", web::post().to(wallet::estimate_withdrawal_fee))
            )
            // WebSocket endpoint
            .route("/ws", web::get().to(super::websocket::ws_handler))
    );
}
