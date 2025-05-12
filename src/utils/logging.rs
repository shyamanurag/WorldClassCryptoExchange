rustuse tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    prelude::*,
};

pub fn init_logger() {
    // Get log level from environment variable or use info as default
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true))
        .init();
}
