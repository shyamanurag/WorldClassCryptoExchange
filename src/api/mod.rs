pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod websocket;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use middleware::{auth::AuthenticationMiddleware, logging::RequestLogger};

pub async fn start_api_server(config: crate::config::Config) -> std::io::Result<()> {
    let server_address = format!("{}:{}", config.api.host, config.api.port);
    
    println!("Starting API server on {}", server_address);
    
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin(&config.api.cors_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .wrap(RequestLogger::new())
            .app_data(web::Data::new(config.clone()))
            // Register API routes
            .configure(routes::register_routes)
    })
    .bind(server_address)?
    .workers(config.api.workers)
    .run()
    .await
}
