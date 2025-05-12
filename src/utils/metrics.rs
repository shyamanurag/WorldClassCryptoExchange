use anyhow::Result;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::net::SocketAddr;
use tokio::task;

use crate::config::Settings;

pub fn init_metrics(config: &Settings) -> Result<PrometheusHandle> {
    let builder = PrometheusBuilder::new();
    
    // Create default Prometheus registry with custom buckets for histograms
    let builder = builder
        .set_buckets_for_metric(
            Matcher::Full("api_request_duration_seconds".to_string()),
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )?
        .set_buckets_for_metric(
            Matcher::Full("order_matching_duration_seconds".to_string()),
            vec![0.000001, 0.000005, 0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01],
        )?;
    
    // Install the recorder and get a handle to the registry
    let (recorder, handle) = builder.build()?;
    metrics::set_boxed_recorder(Box::new(recorder))?;
    
    // Spawn a server to serve the metrics
    let addr = format!("0.0.0.0:{}", determine_metrics_port(&config)).parse::<SocketAddr>()?;
    task::spawn(async move {
        let server = axum::Server::bind(&addr)
            .serve(
                axum::routing::Router::new()
                    .route("/metrics", axum::routing::get(|| async move { handle.render() }))
                    .into_make_service()
            );
        
        if let Err(err) = server.await {
            eprintln!("Metrics server error: {}", err);
        }
    });
    
    Ok(handle)
}

fn determine_metrics_port(config: &Settings) -> u16 {
    // Use a different port for each component to avoid conflicts
    match std::env::var("COMPONENT").unwrap_or_default().as_str() {
        "trading-engine" => config.trading_engine.port + 1000,
        "wallet-system" => config.wallet_system.port + 1000,
        "api-gateway" => config.api_gateway.port + 1000,
        _ => 9090, // Default
    }
}

// Helper functions for recording metrics

// Record a counter metric
pub fn increment_counter(name: &str, value: u64, labels: &[(&str, &str)]) {
    let mut builder = metrics::counter!(name);
    
    for (key, value) in labels {
        builder = builder.label(*key, *value);
    }
    
    builder.increment(value);
}

// Record a gauge metric
pub fn set_gauge(name: &str, value: f64, labels: &[(&str, &str)]) {
    let mut builder = metrics::gauge!(name);
    
    for (key, value) in labels {
        builder = builder.label(*key, *value);
    }
    
    builder.set(value);
}

// Record a histogram metric (for measuring distributions of values)
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, &str)]) {
    let mut builder = metrics::histogram!(name);
    
    for (key, value) in labels {
        builder = builder.label(*key, *value);
    }
    
    builder.record(value);
}

// Measure execution time of a function and record it as a histogram
pub async fn measure_execution_time<F, Fut, R>(name: &str, labels: &[(&str, &str)], f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed().as_secs_f64();
    
    record_histogram(name, duration, labels);
    
    result
}
