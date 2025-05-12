// src/config.rs - Configuration management
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use log::{info, warn};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,
    
    /// Redis connection URL
    pub redis_url: String,
    
    /// JWT secret for authentication
    pub jwt_secret: String,
    
    /// Refresh token secret
    pub refresh_secret: String,
    
    /// Token expiry time in hours
    pub token_expiry_hours: u64,
    
    /// Prefix for metrics
    pub metrics_prefix: String,
    
    /// Log level
    pub log_level: String,
    
    /// API host
    pub api_host: String,
    
    /// API port
    pub api_port: u16,
    
    /// Trading pairs
    pub trading_pairs: Vec<String>,
    
    /// Maximum order value for each trading pair
    pub max_order_values: HashMap<String, f64>,
    
    /// Position limits for each trading pair
    pub position_limits: HashMap<String, f64>,
    
    /// Extra configuration
    pub extra: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://postgres:postgres@localhost:5432/crypto_exchange".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "default_jwt_secret_change_in_production".to_string(),
            refresh_secret: "default_refresh_secret_change_in_production".to_string(),
            token_expiry_hours: 24,
            metrics_prefix: "crypto_exchange".to_string(),
            log_level: "info".to_string(),
            api_host: "0.0.0.0".to_string(),
            api_port: 8080,
            trading_pairs: vec!["BTC-USDT".to_string(), "ETH-USDT".to_string()],
            max_order_values: HashMap::new(),
            position_limits: HashMap::new(),
            extra: HashMap::new(),
        }
    }
}

impl Config {
    /// Load configuration from environment and files
    pub fn load() -> Result<Self> {
        let mut config = Config::default();
        
        // Try to load from config file
        if let Some(config_path) = find_config_file() {
            info!("Loading configuration from file: {:?}", config_path);
            if let Err(e) = load_from_file(&mut config, &config_path) {
                warn!("Failed to load configuration from file: {}", e);
            }
        }
        
        // Load from environment variables (override file settings)
        load_from_env(&mut config);
        
        // Set default max order values if not explicitly set
        if config.max_order_values.is_empty() {
            for pair in &config.trading_pairs {
                config.max_order_values.insert(pair.clone(), 1000000.0);
            }
        }
        
        // Set default position limits if not explicitly set
        if config.position_limits.is_empty() {
            for pair in &config.trading_pairs {
                config.position_limits.insert(pair.clone(), 1000000.0);
            }
        }
        
        // Validate configuration
        validate_config(&config)?;
        
        Ok(config)
    }
}

/// Find the configuration file
fn find_config_file() -> Option<PathBuf> {
    // Check for CONFIG_FILE environment variable
    if let Ok(path) = env::var("CONFIG_FILE") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    
    // Check for .env file in current directory
    let env_path = PathBuf::from(".env");
    if env_path.exists() {
        return Some(env_path);
    }
    
    // Check for config.toml in current directory
    let toml_path = PathBuf::from("config.toml");
    if toml_path.exists() {
        return Some(toml_path);
    }
    
    // Check for config.json in current directory
    let json_path = PathBuf::from("config.json");
    if json_path.exists() {
        return Some(json_path);
    }
    
    // Check home directory
    if let Some(home_dir) = home::home_dir() {
        let home_config = home_dir.join(".worldclass_crypto_exchange").join("config.toml");
        if home_config.exists() {
            return Some(home_config);
        }
    }
    
    None
}

/// Load configuration from environment variables
fn load_from_env(config: &mut Config) {
    if let Ok(url) = env::var("DATABASE_URL") {
        config.database_url = url;
    }
    
    if let Ok(url) = env::var("REDIS_URL") {
        config.redis_url = url;
    }
    
    if let Ok(secret) = env::var("JWT_SECRET") {
        config.jwt_secret = secret;
    }
    
    if let Ok(secret) = env::var("REFRESH_SECRET") {
        config.refresh_secret = secret;
    }
    
    if let Ok(hours) = env::var("TOKEN_EXPIRY_HOURS") {
        if let Ok(hours) = hours.parse() {
            config.token_expiry_hours = hours;
        }
    }
    
    if let Ok(prefix) = env::var("METRICS_PREFIX") {
        config.metrics_prefix = prefix;
    }
    
    if let Ok(level) = env::var("LOG_LEVEL") {
        config.log_level = level;
    }
    
    if let Ok(host) = env::var("API_HOST") {
        config.api_host = host;
    }
    
    if let Ok(port) = env::var("API_PORT") {
        if let Ok(port) = port.parse() {
            config.api_port = port;
        }
    }
    
    if let Ok(pairs) = env::var("TRADING_PAIRS") {
        config.trading_pairs = pairs.split(',').map(|s| s.trim().to_string()).collect();
    }
    
    // Load max order values
    for (key, value) in env::vars() {
        if key.starts_with("MAX_ORDER_VALUE_") {
            let pair = key.replace("MAX_ORDER_VALUE_", "").replace('_', "-");
            if let Ok(value) = value.parse::<f64>() {
                config.max_order_values.insert(pair, value);
            }
        } else if key.starts_with("POSITION_LIMIT_") {
            let pair = key.replace("POSITION_LIMIT_", "").replace('_', "-");
            if let Ok(value) = value.parse::<f64>() {
                config.position_limits.insert(pair, value);
            }
        } else if key.starts_with("CONFIG_") {
            let config_key = key.replace("CONFIG_", "");
            config.extra.insert(config_key, value);
        }
    }
}

/// Load configuration from a file
fn load_from_file(config: &mut Config, path: &Path) -> Result<()> {
    let file = File::open(path).context("Failed to open configuration file")?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line.context("Failed to read line from configuration file")?;
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse key-value pairs
        if let Some(index) = line.find('=') {
            let key = line[..index].trim();
            let value = line[index + 1..].trim();
            
            match key {
                "DATABASE_URL" => config.database_url = value.to_string(),
                "REDIS_URL" => config.redis_url = value.to_string(),
                "JWT_SECRET" => config.jwt_secret = value.to_string(),
                "REFRESH_SECRET" => config.refresh_secret = value.to_string(),
                "TOKEN_EXPIRY_HOURS" => {
                    if let Ok(hours) = value.parse() {
                        config.token_expiry_hours = hours;
                    }
                },
                "METRICS_PREFIX" => config.metrics_prefix = value.to_string(),
                "LOG_LEVEL" => config.log_level = value.to_string(),
                "API_HOST" => config.api_host = value.to_string(),
                "API_PORT" => {
                    if let Ok(port) = value.parse() {
                        config.api_port = port;
                    }
                },
                "TRADING_PAIRS" => {
                    config.trading_pairs = value.split(',').map(|s| s.trim().to_string()).collect();
                },
                _ => {
                    if key.starts_with("MAX_ORDER_VALUE_") {
                        let pair = key.replace("MAX_ORDER_VALUE_", "").replace('_', "-");
                        if let Ok(value) = value.parse::<f64>() {
                            config.max_order_values.insert(pair, value);
                        }
                    } else if key.starts_with("POSITION_LIMIT_") {
                        let pair = key.replace("POSITION_LIMIT_", "").replace('_', "-");
                        if let Ok(value) = value.parse::<f64>() {
                            config.position_limits.insert(pair, value);
                        }
                    } else {
                        // Add to extra configurations
                        config.extra.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Validate the configuration
fn validate_config(config: &Config) -> Result<()> {
    // Check database URL
    if config.database_url.is_empty() {
        return Err(anyhow::anyhow!("Database URL cannot be empty"));
    }
    
    // Check JWT secret
    if config.jwt_secret.is_empty() {
        return Err(anyhow::anyhow!("JWT secret cannot be empty"));
    }
    
    // Check refresh secret
    if config.refresh_secret.is_empty() {
        return Err(anyhow::anyhow!("Refresh secret cannot be empty"));
    }
    
    // Check trading pairs
    if config.trading_pairs.is_empty() {
        return Err(anyhow::anyhow!("Trading pairs cannot be empty"));
    }
    
    Ok(())
}
