use anyhow::{Context, Result};
use deadpool_postgres::{Config, Pool, PoolConfig, Runtime};
use sqlx::postgres::{PgPoolOptions, PgPool};
use std::time::Duration;
use tokio_postgres::NoTls;
use tracing::info;

use crate::config::DatabaseSettings;

// Re-export types for use in other modules
pub use sqlx::postgres::PgRow;
pub use sqlx::{FromRow, Row};

// Using sqlx for database operations with strong types
pub async fn init_db_pool(config: &DatabaseSettings) -> Result<PgPool> {
    info!("Initializing database connection pool with max_connections={}", config.max_connections);
    
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.url)
        .await
        .context("Failed to connect to database")
}

// Using deadpool for connection pooling (alternative approach)
pub fn init_deadpool(config: &DatabaseSettings) -> Result<Pool> {
    info!("Initializing deadpool connection pool with max_connections={}", config.max_connections);
    
    let mut cfg = Config::new();
    cfg.url = Some(config.url.clone());
    cfg.pool = Some(PoolConfig::new(config.max_connections));
    
    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .context("Failed to create database connection pool")
}

// Database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations");
    
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run database migrations")
}

// Helper function to check database health
pub async fn check_database_health(pool: &PgPool) -> Result<bool> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| true)
        .context("Failed to connect to database")
}

// Utility function to create test tables (for development/testing)
#[cfg(feature = "development")]
pub async fn create_test_tables(pool: &PgPool) -> Result<()> {
    info!("Creating test tables for development environment");
    
    // Create a users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            username VARCHAR(50) NOT NULL UNIQUE,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create users table")?;
    
    // Create an assets table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS assets (
            id VARCHAR(10) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            symbol VARCHAR(10) NOT NULL UNIQUE,
            blockchain_id VARCHAR(50) NOT NULL,
            decimals INTEGER NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            min_deposit NUMERIC(28, 8) NOT NULL,
            min_withdrawal NUMERIC(28, 8) NOT NULL,
            withdrawal_fee NUMERIC(28, 8) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create assets table")?;
    
    // Create an orders table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id),
            trading_pair VARCHAR(20) NOT NULL,
            side VARCHAR(4) NOT NULL,
            order_type VARCHAR(20) NOT NULL,
            price NUMERIC(28, 8),
            quantity NUMERIC(28, 8) NOT NULL,
            filled_quantity NUMERIC(28, 8) NOT NULL DEFAULT 0,
            status VARCHAR(20) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CONSTRAINT side_check CHECK (side IN ('buy', 'sell')),
            CONSTRAINT order_type_check CHECK (order_type IN ('market', 'limit', 'stop', 'stop_limit'))
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create orders table")?;
    
    // Create a trades table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trades (
            id UUID PRIMARY KEY,
            trading_pair VARCHAR(20) NOT NULL,
            taker_order_id UUID NOT NULL REFERENCES orders(id),
            maker_order_id UUID NOT NULL REFERENCES orders(id),
            price NUMERIC(28, 8) NOT NULL,
            quantity NUMERIC(28, 8) NOT NULL,
            taker_fee NUMERIC(28, 8) NOT NULL,
            maker_fee NUMERIC(28, 8) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create trades table")?;
    
    // Create a wallets table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wallets (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id),
            asset_id VARCHAR(10) NOT NULL REFERENCES assets(id),
            balance NUMERIC(28, 8) NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, asset_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create wallets table")?;
    
    Ok(())
}

// Model definitions
pub mod models {
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;
    use uuid::Uuid;
    
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct User {
        pub id: Uuid,
        pub username: String,
        pub email: String,
        pub password_hash: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Asset {
        pub id: String,
        pub name: String,
        pub symbol: String,
        pub blockchain_id: String,
        pub decimals: i32,
        pub is_active: bool,
        pub min_deposit: Decimal,
        pub min_withdrawal: Decimal,
        pub withdrawal_fee: Decimal,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Order {
        pub id: Uuid,
        pub user_id: Uuid,
        pub trading_pair: String,
        pub side: String,
        pub order_type: String,
        pub price: Option<Decimal>,
        pub quantity: Decimal,
        pub filled_quantity: Decimal,
        pub status: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Trade {
        pub id: Uuid,
        pub trading_pair: String,
        pub taker_order_id: Uuid,
        pub maker_order_id: Uuid,
        pub price: Decimal,
        pub quantity: Decimal,
        pub taker_fee: Decimal,
        pub maker_fee: Decimal,
        pub created_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Wallet {
        pub id: Uuid,
        pub user_id: Uuid,
        pub asset_id: String,
        pub balance: Decimal,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
}

// Repositories for database operations
pub mod repositories {
    use super::models::{Asset, Order, Trade, User, Wallet};
    use anyhow::{Context, Result};
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use uuid::Uuid;
    
    // User repository
    pub struct UserRepository<'a> {
        pool: &'a PgPool,
    }
    
    impl<'a> UserRepository<'a> {
        pub fn new(pool: &'a PgPool) -> Self {
            Self { pool }
        }
        
        pub async fn create(&self, username: &str, email: &str, password_hash: &str) -> Result<User> {
            let id = Uuid::new_v4();
            
            let user = sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (id, username, email, password_hash)
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#,
            )
            .bind(id)
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .fetch_one(self.pool)
            .await
            .context("Failed to create user")?;
            
            Ok(user)
        }
        
        pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
            let user = sqlx::query_as::<_, User>(
                r#"
                SELECT * FROM users WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(self.pool)
            .await
            .context("Failed to find user by ID")?;
            
            Ok(user)
        }
        
        pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
            let user = sqlx::query_as::<_, User>(
                r#"
                SELECT * FROM users WHERE username = $1
                "#,
            )
            .bind(username)
            .fetch_optional(self.pool)
            .await
            .context("Failed to find user by username")?;
            
            Ok(user)
        }
        
        pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
            let user = sqlx::query_as::<_, User>(
                r#"
                SELECT * FROM users WHERE email = $1
                "#,
            )
            .bind(email)
            .fetch_optional(self.pool)
            .await
            .context("Failed to find user by email")?;
            
            Ok(user)
        }
    }
    
    // Wallet repository
    pub struct WalletRepository<'a> {
        pool: &'a PgPool,
    }
    
    impl<'a> WalletRepository<'a> {
        pub fn new(pool: &'a PgPool) -> Self {
            Self { pool }
        }
        
        pub async fn get_user_wallets(&self, user_id: Uuid) -> Result<Vec<Wallet>> {
            let wallets = sqlx::query_as::<_, Wallet>(
                r#"
                SELECT * FROM wallets WHERE user_id = $1
                "#,
            )
            .bind(user_id)
            .fetch_all(self.pool)
            .await
            .context("Failed to get user wallets")?;
            
            Ok(wallets)
        }
        
        pub async fn get_wallet(&self, user_id: Uuid, asset_id: &str) -> Result<Option<Wallet>> {
            let wallet = sqlx::query_as::<_, Wallet>(
                r#"
                SELECT * FROM wallets WHERE user_id = $1 AND asset_id = $2
                "#,
            )
            .bind(user_id)
            .bind(asset_id)
            .fetch_optional(self.pool)
            .await
            .context("Failed to get wallet")?;
            
            Ok(wallet)
        }
        
        pub async fn create_wallet(&self, user_id: Uuid, asset_id: &str) -> Result<Wallet> {
            let id = Uuid::new_v4();
            
            let wallet = sqlx::query_as::<_, Wallet>(
                r#"
                INSERT INTO wallets (id, user_id, asset_id, balance)
                VALUES ($1, $2, $3, 0)
                RETURNING *
                "#,
            )
            .bind(id)
            .bind(user_id)
            .bind(asset_id)
            .fetch_one(self.pool)
            .await
            .context("Failed to create wallet")?;
            
            Ok(wallet)
        }
        
        pub async fn update_balance(&self, id: Uuid, amount: Decimal) -> Result<Wallet> {
            let wallet = sqlx::query_as::<_, Wallet>(
                r#"
                UPDATE wallets
                SET balance = balance + $1, updated_at = NOW()
                WHERE id = $2
                RETURNING *
                "#,
            )
            .bind(amount)
            .bind(id)
            .fetch_one(self.pool)
            .await
            .context("Failed to update wallet balance")?;
            
            Ok(wallet)
        }
    }
    
    // More repositories can be added here for Order, Trade, etc.
}
