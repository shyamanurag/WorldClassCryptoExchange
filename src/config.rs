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
