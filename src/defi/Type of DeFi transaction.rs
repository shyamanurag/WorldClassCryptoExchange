/// Type of DeFi transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeFiTransactionType {
    Deposit,
    Withdraw,
    Stake,
    Unstake,
    Claim,
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Borrow,
    Repay,
    ClaimRewards,
    Leverage,
    Deleverage,
    FlashLoan,
}

/// Status of a DeFi transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeFiTransactionStatus {
    Pending,
    Confirming,
    Completed,
    Failed,
    Canceled,
}

/// Yield farming position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldFarmPosition {
    pub position_id: PositionId,
    pub rewards_earned: Vec<AssetAmount>,
    pub rewards_claimed: Vec<AssetAmount>,
    pub last_harvest_time: DateTime<Utc>,
    pub strategy: String,
    pub compound_frequency: Option<String>,
    pub auto_compound: bool,
}

/// Liquidity provision position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub position_id: PositionId,
    pub pair: (AssetId, AssetId),
    pub liquidity_token_amount: Decimal,
    pub share_percentage: Decimal,
    pub fees_earned: Vec<AssetAmount>,
}

/// Staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub position_id: PositionId,
    pub asset_staked: AssetId,
    pub amount_staked: Decimal,
    pub rewards_earned: Vec<AssetAmount>,
    pub rewards_claimed: Vec<AssetAmount>,
    pub unstake_time: Option<DateTime<Utc>>,
    pub lock_period_days: Option<u32>,
}

/// Lending position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingPosition {
    pub position_id: PositionId,
    pub asset_supplied: AssetId,
    pub amount_supplied: Decimal,
    pub interest_earned: Decimal,
    pub collateral_factor: Option<Decimal>,
    pub is_collateral: bool,
}

/// Borrowing position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowingPosition {
    pub position_id: PositionId,
    pub asset_borrowed: AssetId,
    pub amount_borrowed: Decimal,
    pub interest_rate: Decimal,
    pub interest_accrued: Decimal,
    pub health_factor: Decimal,
    pub liquidation_threshold: Decimal,
}

/// Interface for protocol adapters
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Get the protocol ID
    fn protocol_id(&self) -> ProtocolId;
    
    /// Get protocol information
    async fn get_protocol_info(&self) -> Result<Protocol>;
    
    /// List available pools
    async fn list_pools(&self) -> Result<Vec<Pool>>;
    
    /// Get APY for a specific pool
    async fn get_pool_apy(&self, pool_id: &PoolId) -> Result<Decimal>;
    
    /// Create a new position
    async fn create_position(
        &self,
        user_id: UserId,
        pool_id: &PoolId,
        assets: Vec<AssetAmount>,
    ) -> Result<(Position, DeFiTransaction)>;
    
    /// Close a position
    async fn close_position(
        &self,
        position_id: &PositionId,
    ) -> Result<(Position, DeFiTransaction)>;
    
    /// Harvest rewards from a position
    async fn harvest_rewards(
        &self,
        position_id: &PositionId,
    ) -> Result<(Position, DeFiTransaction)>;
    
    /// Get current value of a position
    async fn get_position_value(
        &self,
        position_id: &PositionId,
    ) -> Result<Decimal>;
    
    /// Update position data (APY, earned rewards, etc.)
    async fn update_position(
        &self,
        position_id: &PositionId,
    ) -> Result<Position>;
    
    /// Check health of a position (especially for lending/borrowing)
    async fn check_position_health(
        &self,
        position_id: &PositionId,
    ) -> Result<Decimal>;
}

/// Mock implementation of a protocol adapter for Uniswap-like liquidity provision
pub struct UniswapAdapter {
    protocol: Protocol,
    pools: RwLock<Vec<Pool>>,
    positions: RwLock<Vec<Position>>,
    liquidity_positions: RwLock<Vec<LiquidityPosition>>,
    transactions: RwLock<Vec<DeFiTransaction>>,
}

impl UniswapAdapter {
    pub fn new() -> Self {
        let now = Utc::now();
        
        let protocol = Protocol {
            id: "uniswap-v3".to_string(),
            name: "Uniswap V3".to_string(),
            protocol_type: ProtocolType::LiquidityPool,
            chain_id: "ethereum".to_string(),
            contracts: vec![
                ContractInfo {
                    name: "Factory".to_string(),
                    address: "0x1F98431c8aD98523631AE4a59f267346ea31F984".to_string(),
                    abi: "...".to_string(), // Abbreviated
                },
                ContractInfo {
                    name: "Router".to_string(),
                    address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
                    abi: "...".to_string(), // Abbreviated
                },
            ],
            description: "Uniswap V3 is a concentrated liquidity AMM".to_string(),
            risk_level: RiskLevel::Medium,
            tvl: Decimal::new(500000000, 0), // $500M
            apy_range: (Decimal::new(5, 0), Decimal::new(100, 0)), // 5-100% APY
            is_active: true,
            created_at: now,
            updated_at: now,
        };
        
        UniswapAdapter {
            protocol,
            pools: RwLock::new(Vec::new()),
            positions: RwLock::new(Vec::new()),
            liquidity_positions: RwLock::new(Vec::new()),
            transactions: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn initialize_pools(&self) -> Result<()> {
        let now = Utc::now();
        
        let pools = vec![
            Pool {
                id: "uniswap-v3-eth-usdt-500".to_string(),
                protocol_id: self.protocol.id.clone(),
                name: "ETH-USDT 0.05%".to_string(),
                assets: vec!["ETH".to_string(), "USDT".to_string()],
                tvl: Decimal::new(100000000, 0), // $100M
                apy: Decimal::new(25, 0), // 25% APY
                fees: PoolFees {
                    entry_fee: Decimal::new(0, 0),
                    exit_fee: Decimal::new(0, 0),
                    performance_fee: Decimal::new(0, 0),
                    management_fee: Decimal::new(5, 3), // 0.5% fee
                },
                is_active: true,
                risk_level: RiskLevel::Medium,
                created_at: now,
                updated_at: now,
            },
            Pool {
                id: "uniswap-v3-btc-usdt-500".to_string(),
                protocol_id: self.protocol.id.clone(),
                name: "BTC-USDT 0.05%".to_string(),
                assets: vec!["BTC".to_string(), "USDT".to_string()],
                tvl: Decimal::new(150000000, 0), // $150M
                apy: Decimal::new(20, 0), // 20% APY
                fees: PoolFees {
                    entry_fee: Decimal::new(0, 0),
                    exit_fee: Decimal::new(0, 0),
                    performance_fee: Decimal::new(0, 0),
                    management_fee: Decimal::new(5, 3), // 0.5% fee
                },
                is_active: true,
                risk_level: RiskLevel::Medium,
                created_at: now,
                updated_at: now,
            },
            Pool {
                id: "uniswap-v3-eth-usdt-3000".to_string(),
                protocol_id: self.protocol.id.clone(),
                name: "ETH-USDT 0.3%".to_string(),
                assets: vec!["ETH".to_string(), "USDT".to_string()],
                tvl: Decimal::new(80000000, 0), // $80M
                apy: Decimal::new(40, 0), // 40% APY
                fees: PoolFees {
                    entry_fee: Decimal::new(0, 0),
                    exit_fee: Decimal::new(0, 0),
                    performance_fee: Decimal::new(0, 0),
                    management_fee: Decimal::new(3, 2), // 3.0% fee
                },
                is_active: true,
                risk_level: RiskLevel::Medium,
                created_at: now,
                updated_at: now,
            },
        ];
        
        let mut pools_lock = self.pools.write().await;
        *pools_lock = pools;
        
        Ok(())
    }
    
    async fn find_pool(&self, pool_id: &PoolId) -> Result<Pool> {
        let pools = self.pools.read().await;
        pools
            .iter()
            .find(|p| p.id == *pool_id)
            .cloned()
            .ok_or_else(|| anyhow!("Pool not found: {}", pool_id))
    }
    
    async fn find_position(&self, position_id: &PositionId) -> Result<Position> {
        let positions = self.positions.read().await;
        positions
            .iter()
            .find(|p| p.id == *position_id)
            .cloned()
            .ok_or_else(|| anyhow!("Position not found: {}", position_id))
    }
    
    async fn find_liquidity_position(&self, position_id: &PositionId) -> Result<LiquidityPosition> {
        let liquidity_positions = self.liquidity_positions.read().await;
        liquidity_positions
            .iter()
            .find(|p| p.position_id == *position_id)
            .cloned()
            .ok_or_else(|| anyhow!("Liquidity position not found: {}", position_id))
    }
}

#[async_trait]
impl ProtocolAdapter for UniswapAdapter {
    fn protocol_id(&self) -> ProtocolId {
        self.protocol.id.clone()
    }
    
    async fn get_protocol_info(&self) -> Result<Protocol> {
        Ok(self.protocol.clone())
    }
    
    async fn list_pools(&self) -> Result<Vec<Pool>> {
        let pools = self.pools.read().await;
        Ok(pools.clone())
    }
    
    async fn get_pool_apy(&self, pool_id: &PoolId) -> Result<Decimal> {
        let pool = self.find_pool(pool_id).await?;
        Ok(pool.apy)
    }
    
    async fn create_position(
        &self,
        user_id: UserId,
        pool_id: &PoolId,
        assets: Vec<AssetAmount>,
    ) -> Result<(Position, DeFiTransaction)> {
        let pool = self.find_pool(pool_id).await?;
        let now = Utc::now();
        
        // Validate assets match pool assets
        let pool_assets: std::collections::HashSet<_> = pool.assets.iter().cloned().collect();
        let provided_assets: std::collections::HashSet<_> = assets.iter().map(|a| a.asset_id.clone()).collect();
        
        if !provided_assets.is_subset(&pool_assets) {
            return Err(anyhow!("Provided assets do not match pool assets"));
        }
        
        // Calculate value in USD (mock implementation)
        let value_usd = assets.iter().fold(Decimal::ZERO, |acc, asset| {
            // In a real implementation, we'd get the USD price from an oracle
            let asset_price = match asset.asset_id.as_str() {
                "BTC" => Decimal::new(50000, 0),  // $50,000 per BTC
                "ETH" => Decimal::new(3000, 0),   // $3,000 per ETH
                "USDT" => Decimal::new(1, 0),     // $1 per USDT
                _ => Decimal::new(1, 0),          // Default $1
            };
            
            acc + (asset.amount * asset_price)
        });
        
        // Calculate liquidity tokens (simplified)
        let liquidity_token_amount = value_usd / Decimal::new(10, 0); // Arbitrary calculation
        
        // Calculate share percentage
        let share_percentage = (value_usd * Decimal::new(100, 0)) / pool.tvl;
        
        // Create position
        let position_id = Uuid::new_v4();
        let position = Position {
            id: position_id,
            user_id,
            protocol_id: self.protocol.id.clone(),
            pool_id: Some(pool.id.clone()),
            assets_deposited: assets.clone(),
            tokens_received: vec![
                AssetAmount {
                    asset_id: format!("{}-LP", pool.id),
                    amount: liquidity_token_amount,
                },
            ],
            value_usd,
            apy: pool.apy,
            status: PositionStatus::Active,
            created_at: now,
            updated_at: now,
            closed_at: None,
        };
        
        // Create liquidity position
        let liquidity_position = LiquidityPosition {
            position_id,
            pair: (pool.assets[0].clone(), pool.assets[1].clone()),
            liquidity_token_amount,
            share_percentage,
            fees_earned: Vec::new(),
        };
        
        // Create transaction
        let transaction = DeFiTransaction {
            id: Uuid::new_v4(),
            user_id,
            protocol_id: self.protocol.id.clone(),
            pool_id: Some(pool.id.clone()),
            position_id: Some(position_id),
            transaction_type: DeFiTransactionType::AddLiquidity,
            assets_in: assets,
            assets_out: vec![
                AssetAmount {
                    asset_id: format!("{}-LP", pool.id),
                    amount: liquidity_token_amount,
                },
            ],
            value_usd,
            blockchain_txid: Some(format!("0x{}", Uuid::new_v4().to_string().replace("-", ""))),
            status: DeFiTransactionStatus::Completed,
            created_at: now,
            updated_at: now,
            completed_at: Some(now),
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            positions.push(position.clone());
        }
        
        {
            let mut liquidity_positions = self.liquidity_positions.write().await;
            liquidity_positions.push(liquidity_position);
        }
        
        {
            let mut transactions = self.transactions.write().await;
            transactions.push(transaction.clone());
        }
        
        Ok((position, transaction))
    }
    
    async fn close_position(
        &self,
        position_id: &PositionId,
    ) -> Result<(Position, DeFiTransaction)> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Err(anyhow!("Position is not active"));
        }
        
        let liquidity_position = self.find_liquidity_position(position_id).await?;
        let pool_id = position.pool_id.clone().ok_or_else(|| anyhow!("Pool ID missing"))?;
        let pool = self.find_pool(&pool_id).await?;
        
        let now = Utc::now();
        
        // Calculate fees earned (simplified mock implementation)
        let time_in_pool = (now - position.created_at).num_days() as f64;
        let daily_fee_rate = (pool.apy.to_f64().unwrap_or(0.0) / 365.0) / 100.0;
        let fee_multiplier = 1.0 + (time_in_pool * daily_fee_rate);
        
        let value_with_fees = (position.value_usd.to_f64().unwrap_or(0.0) * fee_multiplier) as f64;
        let fees_earned = value_with_fees - position.value_usd.to_f64().unwrap_or(0.0);
        
        // Calculate returned assets with fees
        let mut assets_out = Vec::new();
        
        // Distribute fees proportionally to the assets
        let total_input_value = position.assets_deposited.iter().fold(Decimal::ZERO, |acc, asset| {
            acc + asset.amount
        });
        
        for asset in &position.assets_deposited {
            let proportion = asset.amount / total_input_value;
            let fee_share = Decimal::from_f64(fees_earned).unwrap_or(Decimal::ZERO) * proportion;
            
            assets_out.push(AssetAmount {
                asset_id: asset.asset_id.clone(),
                amount: asset.amount + fee_share,
            });
        }
        
        // Create transaction
        let transaction = DeFiTransaction {
            id: Uuid::new_v4(),
            user_id: position.user_id,
            protocol_id: position.protocol_id.clone(),
            pool_id: position.pool_id.clone(),
            position_id: Some(*position_id),
            transaction_type: DeFiTransactionType::RemoveLiquidity,
            assets_in: position.tokens_received.clone(),
            assets_out: assets_out.clone(),
            value_usd: Decimal::from_f64(value_with_fees).unwrap_or(Decimal::ZERO),
            blockchain_txid: Some(format!("0x{}", Uuid::new_v4().to_string().replace("-", ""))),
            status: DeFiTransactionStatus::Completed,
            created_at: now,
            updated_at: now,
            completed_at: Some(now),
        };
        
        // Update position
        let updated_position = Position {
            status: PositionStatus::Closed,
            updated_at: now,
            closed_at: Some(now),
            ..position.clone()
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            if let Some(pos) = positions.iter_mut().find(|p| p.id == *position_id) {
                *pos = updated_position.clone();
            }
        }
        
        {
            let mut transactions = self.transactions.write().await;
            transactions.push(transaction.clone());
        }
        
        Ok((updated_position, transaction))
    }
    
    async fn harvest_rewards(
        &self,
        position_id: &PositionId,
    ) -> Result<(Position, DeFiTransaction)> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Err(anyhow!("Position is not active"));
        }
        
        let liquidity_position = self.find_liquidity_position(position_id).await?;
        let pool_id = position.pool_id.clone().ok_or_else(|| anyhow!("Pool ID missing"))?;
        let pool = self.find_pool(&pool_id).await?;
        
        let now = Utc::now();
        
        // Calculate rewards (simplified mock implementation)
        let time_since_creation = (now - position.created_at).num_days() as f64;
        let daily_reward_rate = (pool.apy.to_f64().unwrap_or(0.0) / 365.0) / 100.0;
        let reward_multiplier = time_since_creation * daily_reward_rate;
        
        let rewards_value = position.value_usd.to_f64().unwrap_or(0.0) * reward_multiplier;
        
        // Create rewards - for simplicity, we'll use the first asset in the pool
        let reward_asset = pool.assets[0].clone();
        let reward_amount = Decimal::from_f64(rewards_value).unwrap_or(Decimal::ZERO);
        
        let rewards = vec![
            AssetAmount {
                asset_id: reward_asset.clone(),
                amount: reward_amount,
            },
        ];
        
        // Create transaction
        let transaction = DeFiTransaction {
            id: Uuid::new_v4(),
            user_id: position.user_id,
            protocol_id: position.protocol_id.clone(),
            pool_id: position.pool_id.clone(),
            position_id: Some(*position_id),
            transaction_type: DeFiTransactionType::ClaimRewards,
            assets_in: Vec::new(),
            assets_out: rewards.clone(),
            value_usd: reward_amount,
            blockchain_txid: Some(format!("0x{}", Uuid::new_v4().to_string().replace("-", ""))),
            status: DeFiTransactionStatus::Completed,
            created_at: now,
            updated_at: now,
            completed_at: Some(now),
        };
        
        // Position remains active after harvesting
        let updated_position = Position {
            updated_at: now,
            ..position.clone()
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            if let Some(pos) = positions.iter_mut().find(|p| p.id == *position_id) {
                *pos = updated_position.clone();
            }
        }
        
        {
            let mut transactions = self.transactions.write().await;
            transactions.push(transaction.clone());
        }
        
        Ok((updated_position, transaction))
    }
    
    async fn get_position_value(
        &self,
        position_id: &PositionId,
    ) -> Result<Decimal> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Ok(Decimal::ZERO);
        }
        
        let now = Utc::now();
        let time_in_pool = (now - position.created_at).num_days() as f64;
        let daily_growth_rate = (position.apy.to_f64().unwrap_or(0.0) / 365.0) / 100.0;
        let value_multiplier = 1.0 + (time_in_pool * daily_growth_rate);
        
        let current_value = position.value_usd.to_f64().unwrap_or(0.0) * value_multiplier;
        Ok(Decimal::from_f64(current_value).unwrap_or(Decimal::ZERO))
    }
    
    async fn update_position(
        &self,
        position_id: &PositionId,
    ) -> Result<Position> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Ok(position);
        }
        
        let current_value = self.get_position_value(position_id).await?;
        
        // Update position with current value
        let updated_position = Position {
            value_usd: current_value,
            updated_at: Utc::now(),
            ..position
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            if let Some(pos) = positions.iter_mut().find(|p| p.id == *position_id) {
                *pos = updated_position.clone();
            }
        }
        
        Ok(updated_position)
    }
    
    async fn check_position_health(
        &self,
        position_id: &PositionId,
    ) -> Result<Decimal> {
        // For liquidity positions, health is always 1.0 (healthy)
        // This would be different for lending/borrowing positions
        Ok(Decimal::new(1, 0))
    }
}

/// Mock implementation of a protocol adapter for Aave-like lending protocol
pub struct AaveLendingAdapter {
    protocol: Protocol,
    pools: RwLock<Vec<Pool>>,
    positions: RwLock<Vec<Position>>,
    lending_positions: RwLock<Vec<LendingPosition>>,
    borrowing_positions: RwLock<Vec<BorrowingPosition>>,
    transactions: RwLock<Vec<DeFiTransaction>>,
}

impl AaveLendingAdapter {
    pub fn new() -> Self {
        let now = Utc::now();
        
        let protocol = Protocol {
            id: "aave-v3".to_string(),
            name: "Aave V3".to_string(),
            protocol_type: ProtocolType::LendingProtocol,
            chain_id: "ethereum".to_string(),
            contracts: vec![
                ContractInfo {
                    name: "LendingPool".to_string(),
                    address: "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9".to_string(),
                    abi: "...".to_string(), // Abbreviated
                },
                ContractInfo {
                    name: "LendingPoolAddressesProvider".to_string(),
                    address: "0xB53C1a33016B2DC2fF3653530bfF1848a515c8c5".to_string(),
                    abi: "...".to_string(), // Abbreviated
                },
            ],
            description: "Aave V3 is a decentralized lending protocol".to_string(),
            risk_level: RiskLevel::Medium,
            tvl: Decimal::new(8000000000, 0), // $8B
            apy_range: (Decimal::new(1, 0), Decimal::new(15, 0)), // 1-15% APY
            is_active: true,
            created_at: now,
            updated_at: now,
        };
        
        AaveLendingAdapter {
            protocol,
            pools: RwLock::new(Vec::new()),
            positions: RwLock::new(Vec::new()),
            lending_positions: RwLock::new(Vec::new()),
            borrowing_positions: RwLock::new(Vec::new()),
            transactions: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn initialize_pools(&self) -> Result<()> {
        let now = Utc::now();
        
        let pools = vec![
            Pool {
                id: "aave-v3-eth-market".to_string(),
                protocol_id: self.protocol.id.clone(),
                name: "ETH Market".to_string(),
                assets: vec!["ETH".to_string()],
                tvl: Decimal::new(2000000000, 0), // $2B
                apy: Decimal::new(2, 0), // 2% APY for lenders
                fees: PoolFees {
                    entry_fee: Decimal::new(0, 0),
                    exit_fee: Decimal::new(0, 0),
                    performance_fee: Decimal::new(0, 0),
                    management_fee: Decimal::new(0, 0),
                },
                is_active: true,
                risk_level: RiskLevel::Medium,
                created_at: now,
                updated_at: now,
            },
            Pool {
                id: "aave-v3-btc-market".to_string(),
                protocol_id: self.protocol.id.clone(),
                name: "BTC Market".to_string(),
                assets: vec!["BTC".to_string()],
                tvl: Decimal::new(1500000000, 0), // $1.5B
                apy: Decimal::new(15, 1), // 1.5% APY for lenders
                fees: PoolFees {
                    entry_fee: Decimal::new(0, 0),
                    exit_fee: Decimal::new(0, 0),
                    performance_fee: Decimal::new(0, 0),
                    management_fee: Decimal::new(0, 0),
                },
                is_active: true,
                risk_level: RiskLevel::Medium,
                created_at: now,
                updated_at: now,
            },
            Pool {
                id: "aave-v3-usdt-market".to_string(),
                protocol_id: self.protocol.id.clone(),
                name: "USDT Market".to_string(),
                assets: vec!["USDT".to_string()],
                tvl: Decimal::new(3000000000, 0), // $3B
                apy: Decimal::new(35, 1), // 3.5% APY for lenders
                fees: PoolFees {
                    entry_fee: Decimal::new(0, 0),
                    exit_fee: Decimal::new(0, 0),
                    performance_fee: Decimal::new(0, 0),
                    management_fee: Decimal::new(0, 0),
                },
                is_active: true,
                risk_level: RiskLevel::Low,
                created_at: now,
                updated_at: now,
            },
        ];
        
        let mut pools_lock = self.pools.write().await;
        *pools_lock = pools;
        
        Ok(())
    }
    
    async fn find_pool(&self, pool_id: &PoolId) -> Result<Pool> {
        let pools = self.pools.read().await;
        pools
            .iter()
            .find(|p| p.id == *pool_id)
            .cloned()
            .ok_or_else(|| anyhow!("Pool not found: {}", pool_id))
    }
    
    async fn find_position(&self, position_id: &PositionId) -> Result<Position> {
        let positions = self.positions.read().await;
        positions
            .iter()
            .find(|p| p.id == *position_id)
            .cloned()
            .ok_or_else(|| anyhow!("Position not found: {}", position_id))
    }
    
    async fn find_lending_position(&self, position_id: &PositionId) -> Result<LendingPosition> {
        let lending_positions = self.lending_positions.read().await;
        lending_positions
            .iter()
            .find(|p| p.position_id == *position_id)
            .cloned()
            .ok_or_else(|| anyhow!("Lending position not found: {}", position_id))
    }
    
    pub async fn create_borrow_position(
        &self,
        user_id: UserId,
        collateral_position_id: &PositionId,
        borrow_pool_id: &PoolId,
        borrow_amount: Decimal,
    ) -> Result<(Position, BorrowingPosition, DeFiTransaction)> {
        // First, verify the collateral position
        let collateral_position = self.find_position(collateral_position_id).await?;
        let lending_position = self.find_lending_position(collateral_position_id).await?;
        
        if collateral_position.status != PositionStatus::Active {
            return Err(anyhow!("Collateral position is not active"));
        }
        
        if !lending_position.is_collateral {
            return Err(anyhow!("Position is not marked as collateral"));
        }
        
        // Get pool for borrowing
        let borrow_pool = self.find_pool(borrow_pool_id).await?;
        let borrow_asset = borrow_pool.assets[0].clone();
        
        // Calculate collateral value and borrowing power
        let collateral_value = collateral_position.value_usd;
        let collateral_factor = lending_position.collateral_factor.unwrap_or(Decimal::new(75, 2)); // 0.75
        let max_borrow_value = collateral_value * collateral_factor;
        
        // Check if borrow amount is within limits
        let borrow_asset_price = match borrow_asset.as_str() {
            "BTC" => Decimal::new(50000, 0),  // $50,000 per BTC
            "ETH" => Decimal::new(3000, 0),   // $3,000 per ETH
            "USDT" => Decimal::new(1, 0),     // $1 per USDT
            _ => Decimal::new(1, 0),          // Default $1
        };
        
        let borrow_value = borrow_amount * borrow_asset_price;
        
        if borrow_value > max_borrow_value {
            return Err(anyhow!("Borrow amount exceeds maximum allowed"));
        }
        
        let now = Utc::now();
        
        // Get borrow interest rate (simplified)
        let borrow_interest_rate = Decimal::new(5, 0); // 5% APY
        
        // Calculate health factor
        let health_factor = collateral_value / borrow_value;
        
        // Create borrowing position
        let position_id = Uuid::new_v4();
        let position = Position {
            id: position_id,
            user_id,
            protocol_id: self.protocol.id.clone(),
            pool_id: Some(borrow_pool.id.clone()),
            assets_deposited: Vec::new(),
            tokens_received: vec![
                AssetAmount {
                    asset_id: borrow_asset.clone(),
                    amount: borrow_amount,
                },
            ],
            value_usd: borrow_value,
            apy: borrow_interest_rate,
            status: PositionStatus::Active,
            created_at: now,
            updated_at: now,
            closed_at: None,
        };
        
        let borrowing_position = BorrowingPosition {
            position_id,
            asset_borrowed: borrow_asset.clone(),
            amount_borrowed: borrow_amount,
            interest_rate: borrow_interest_rate,
            interest_accrued: Decimal::ZERO,
            health_factor,
            liquidation_threshold: Decimal::new(12, 1), // 1.2
        };
        
        // Create transaction
        let transaction = DeFiTransaction {
            id: Uuid::new_v4(),
            user_id,
            protocol_id: self.protocol.id.clone(),
            pool_id: Some(borrow_pool.id.clone()),
            position_id: Some(position_id),
            transaction_type: DeFiTransactionType::Borrow,
            assets_in: Vec::new(),
            assets_out: vec![
                AssetAmount {
                    asset_id: borrow_asset,
                    amount: borrow_amount,
                },
            ],
            value_usd: borrow_value,
            blockchain_txid: Some(format!("0x{}", Uuid::new_v4().to_string().replace("-", ""))),
            status: DeFiTransactionStatus::Completed,
            created_at: now,
            updated_at: now,
            completed_at: Some(now),
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            positions.push(position.clone());
        }
        
        {
            let mut borrowing_positions = self.borrowing_positions.write().await;
            borrowing_positions.push(borrowing_position.clone());
        }
        
        {
            let mut transactions = self.transactions.write().await;
            transactions.push(transaction.clone());
        }
        
        Ok((position, borrowing_position, transaction))
    }
}

#[async_trait]
impl ProtocolAdapter for AaveLendingAdapter {
    fn protocol_id(&self) -> ProtocolId {
        self.protocol.id.clone()
    }
    
    async fn get_protocol_info(&self) -> Result<Protocol> {
        Ok(self.protocol.clone())
    }
    
    async fn list_pools(&self) -> Result<Vec<Pool>> {
        let pools = self.pools.read().await;
        Ok(pools.clone())
    }
    
    async fn get_pool_apy(&self, pool_id: &PoolId) -> Result<Decimal> {
        let pool = self.find_pool(pool_id).await?;
        Ok(pool.apy)
    }
    
    async fn create_position(
        &self,
        user_id: UserId,
        pool_id: &PoolId,
        assets: Vec<AssetAmount>,
    ) -> Result<(Position, DeFiTransaction)> {
        let pool = self.find_pool(pool_id).await?;
        let now = Utc::now();
        
        // Validate assets match pool assets
        if assets.len() != 1 || assets[0].asset_id != pool.assets[0] {
            return Err(anyhow!("Provided assets do not match pool assets"));
        }
        
        let asset = &assets[0];
        
        // Calculate value in USD (mock implementation)
        let asset_price = match asset.asset_id.as_str() {
            "BTC" => Decimal::new(50000, 0),  // $50,000 per BTC
            "ETH" => Decimal::new(3000, 0),   // $3,000 per ETH
            "USDT" => Decimal::new(1, 0),     // $1 per USDT
            _ => Decimal::new(1, 0),          // Default $1
        };
        
        let value_usd = asset.amount * asset_price;
        
        // Calculate aToken amount (in Aave, 1:1 with deposited asset)
        let atoken_amount = asset.amount;
        
        // Create position
        let position_id = Uuid::new_v4();
        let position = Position {
            id: position_id,
            user_id,
            protocol_id: self.protocol.id.clone(),
            pool_id: Some(pool.id.clone()),
            assets_deposited: assets.clone(),
            tokens_received: vec![
                AssetAmount {
                    asset_id: format!("a{}", asset.asset_id),
                    amount: atoken_amount,
                },
            ],
            value_usd,
            apy: pool.apy,
            status: PositionStatus::Active,
            created_at: now,
            updated_at: now,
            closed_at: None,
        };
        
        // Create lending position
        let lending_position = LendingPosition {
            position_id,
            asset_supplied: asset.asset_id.clone(),
            amount_supplied: asset.amount,
            interest_earned: Decimal::ZERO,
            collateral_factor: Some(Decimal::new(75, 2)), // 0.75 (75%)
            is_collateral: true,
        };
        
        // Create transaction
        let transaction = DeFiTransaction {
            id: Uuid::new_v4(),
            user_id,
            protocol_id: self.protocol.id.clone(),
            pool_id: Some(pool.id.clone()),
            position_id: Some(position_id),
            transaction_type: DeFiTransactionType::Deposit,
            assets_in: assets,
            assets_out: vec![
                AssetAmount {
                    asset_id: format!("a{}", asset.asset_id),
                    amount: atoken_amount,
                },
            ],
            value_usd,
            blockchain_txid: Some(format!("0x{}", Uuid::new_v4().to_string().replace("-", ""))),
            status: DeFiTransactionStatus::Completed,
            created_at: now,
            updated_at: now,
            completed_at: Some(now),
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            positions.push(position.clone());
        }
        
        {
            let mut lending_positions = self.lending_positions.write().await;
            lending_positions.push(lending_position);
        }
        
        {
            let mut transactions = self.transactions.write().await;
            transactions.push(transaction.clone());
        }
        
        Ok((position, transaction))
    }
    
    async fn close_position(
        &self,
        position_id: &PositionId,
    ) -> Result<(Position, DeFiTransaction)> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Err(anyhow!("Position is not active"));
        }
        
        // Check if this is a lending position
        let lending_position = match self.lending_positions.read().await.iter().find(|p| p.position_id == *position_id) {
            Some(p) => p.clone(),
            None => return Err(anyhow!("Not a lending position")),
        };
        
        // Calculate interest earned
        let now = Utc::now();
        let days_active = (now - position.created_at).num_days() as f64;
        let annual_interest_rate = position.apy.to_f64().unwrap_or(0.0) / 100.0;
        let interest_earned = lending_position.amount_supplied.to_f64().unwrap_or(0.0) * annual_interest_rate * (days_active / 365.0);
        
        let interest_amount = Decimal::from_f64(interest_earned).unwrap_or(Decimal::ZERO);
        let total_amount = lending_position.amount_supplied + interest_amount;
        
        // Calculate current value
        let asset_price = match lending_position.asset_supplied.as_str() {
            "BTC" => Decimal::new(50000, 0),  // $50,000 per BTC
            "ETH" => Decimal::new(3000, 0),   // $3,000 per ETH
            "USDT" => Decimal::new(1, 0),     // $1 per USDT
            _ => Decimal::new(1, 0),          // Default $1
        };
        
        let value_usd = total_amount * asset_price;
        
        // Create transaction
        let transaction = DeFiTransaction {
            id: Uuid::new_v4(),
            user_id: position.user_id,
            protocol_id: position.protocol_id.clone(),
            pool_id: position.pool_id.clone(),
            position_id: Some(*position_id),
            transaction_type: DeFiTransactionType::Withdraw,
            assets_in: position.tokens_received.clone(),
            assets_out: vec![
                AssetAmount {
                    asset_id: lending_position.asset_supplied.clone(),
                    amount: total_amount,
                },
            ],
            value_usd,
            blockchain_txid: Some(format!("0x{}", Uuid::new_v4().to_string().replace("-", ""))),
            status: DeFiTransactionStatus::Completed,
            created_at: now,
            updated_at: now,
            completed_at: Some(now),
        };
        
        // Update position
        let updated_position = Position {
            status: PositionStatus::Closed,
            updated_at: now,
            closed_at: Some(now),
            ..position.clone()
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            if let Some(pos) = positions.iter_mut().find(|p| p.id == *position_id) {
                *pos = updated_position.clone();
            }
        }
        
        {
            let mut transactions = self.transactions.write().await;
            transactions.push(transaction.clone());
        }
        
        Ok((updated_position, transaction))
    }
    
    async fn harvest_rewards(
        &self,
        position_id: &PositionId,
    ) -> Result<(Position, DeFiTransaction)> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Err(anyhow!("Position is not active"));
        }
        
        // Aave doesn't have harvestable rewards, it automatically compounds interest
        return Err(anyhow!("Operation not supported for this protocol"));
    }
    
    async fn get_position_value(
        &self,
        position_id: &PositionId,
    ) -> Result<Decimal> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Ok(Decimal::ZERO);
        }
        
        // Check if this is a lending position
        if let Some(lending_position) = self.lending_positions.read().await.iter().find(|p| p.position_id == *position_id) {
            // Calculate interest earned
            let now = Utc::now();
            let days_active = (now - position.created_at).num_days() as f64;
            let annual_interest_rate = position.apy.to_f64().unwrap_or(0.0) / 100.0;
            let interest_earned = lending_position.amount_supplied.to_f64().unwrap_or(0.0) * annual_interest_rate * (days_active / 365.0);
            
            let interest_amount = Decimal::from_f64(interest_earned).unwrap_or(Decimal::ZERO);
            let total_amount = lending_position.amount_supplied + interest_amount;
            
            // Calculate current value
            let asset_price = match lending_position.asset_supplied.as_str() {
                "BTC" => Decimal::new(50000, 0),  // $50,000 per BTC
                "ETH" => Decimal::new(3000, 0),   // $3,000 per ETH
                "USDT" => Decimal::new(1, 0),     // $1 per USDT
                _ => Decimal::new(1, 0),          // Default $1
            };
            
            return Ok(total_amount * asset_price);
        }
        
        // Check if this is a borrowing position
        if let Some(borrowing_position) = self.borrowing_positions.read().await.iter().find(|p| p.position_id == *position_id) {
            // Calculate interest accrued
            let now = Utc::now();
            let days_active = (now - position.created_at).num_days() as f64;
            let annual_interest_rate = borrowing_position.interest_rate.to_f64().unwrap_or(0.0) / 100.0;
            let interest_accrued = borrowing_position.amount_borrowed.to_f64().unwrap_or(0.0) * annual_interest_rate * (days_active / 365.0);
            
            let interest_amount = Decimal::from_f64(interest_accrued).unwrap_or(Decimal::ZERO);
            let total_amount = borrowing_position.amount_borrowed + interest_amount;
            
            // Calculate current value
            let asset_price = match borrowing_position.asset_borrowed.as_str() {
                "BTC" => Decimal::new(50000, 0),  // $50,000 per BTC
                "ETH" => Decimal::new(3000, 0),   // $3,000 per ETH
                "USDT" => Decimal::new(1, 0),     // $1 per USDT
                _ => Decimal::new(1, 0),          // Default $1
            };
            
            return Ok(total_amount * asset_price);
        }
        
        Ok(position.value_usd)
    }
    
    async fn update_position(
        &self,
        position_id: &PositionId,
    ) -> Result<Position> {
        let position = self.find_position(position_id).await?;
        
        if position.status != PositionStatus::Active {
            return Ok(position);
        }
        
        let current_value = self.get_position_value(position_id).await?;
        
        // Update position with current value
        let updated_position = Position {
            value_usd: current_value,
            updated_at: Utc::now(),
            ..position
        };
        
        // Update state
        {
            let mut positions = self.positions.write().await;
            if let Some(pos) = positions.iter_mut().find(|p| p.id == *position_id) {
                *pos = updated_position.clone();
            }
        }
        
        Ok(updated_position)
    }
    
    async fn check_position_health(
        &self,
        position_id: &PositionId,
    ) -> Result<Decimal> {
        // Check if this is a borrowing position
        if let Some(borrowing_position) = self.borrowing_positions.read().await.iter().find(|p| p.position_id == *position_id) {
            return Ok(borrowing_position.health_factor);
        }
        
        // For lending positions, health is always 1.0 (healthy)
        Ok(Decimal::new(1, 0))
    }
}

/// DeFi protocol registry
pub struct ProtocolRegistry {
    adapters: RwLock<HashMap<ProtocolId, Arc<dyn ProtocolAdapter + Send + Sync>>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        ProtocolRegistry {
            adapters: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn register_adapter(&self, adapter: Arc<dyn ProtocolAdapter + Send + Sync>) -> Result<()> {
        let protocol_id = adapter.protocol_id();
        let mut adapters = self.adapters.write().await;
        
        if adapters.contains_key(&protocol_id) {
            return Err(anyhow!("Protocol already registered: {}", protocol_id));
        }
        
        adapters.insert(protocol_id, adapter);
        Ok(())
    }
    
    pub async fn get_adapter(&self, protocol_id: &ProtocolId) -> Result<Arc<dyn ProtocolAdapter + Send + Sync>> {
        let adapters = self.adapters.read().await;
        adapters
            .get(protocol_id)
            .cloned()
            .ok_or_else(|| anyhow!("Protocol not found: {}", protocol_id))
    }
    
    pub async fn list_protocols(&self) -> Result<Vec<Protocol>> {
        let adapters = self.adapters.read().await;
        let mut protocols = Vec::new();
        
        for adapter in adapters.values() {
            protocols.push(adapter.get_protocol_info().await?);
        }
        
        Ok(protocols)
    }
}

/// DeFi integration manager
pub struct DeFiManager {
    registry: Arc<ProtocolRegistry>,
    user_positions: RwLock<HashMap<UserId, Vec<PositionId>>>,
}

impl DeFiManager {
    pub fn new(registry: Arc<ProtocolRegistry>) -> Self {
        DeFiManager {
            registry,
            user_positions: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn list_protocols(&self) -> Result<Vec<Protocol>> {
        self.registry.list_protocols().await
    }
    
    pub async fn list_pools(&self, protocol_id: &ProtocolId) -> Result<Vec<Pool>> {
        let adapter = self.registry.get_adapter(protocol_id).await?;
        adapter.list_pools().await
    }
    
    pub async fn create_position(
        &self,
        user_id: UserId,
        protocol_id: &ProtocolId,
        pool_id: &PoolId,
        assets: Vec<AssetAmount>,
    ) -> Result<(Position, DeFiTransaction)> {
        let adapter = self.registry.get_adapter(protocol_id).await?;
        
        let (position, transaction) = adapter.create_position(user_id, pool_id, assets).await?;
        
        // Track user positions
        {
            let mut user_positions = self.user_positions.write().await;
            user_positions
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(position.id);
        }
        
        Ok((position, transaction))
    }
    
    pub async fn close_position(
        &self,
        user_id: UserId,
        position_id: PositionId,
    ) -> Result<(Position, DeFiTransaction)> {
        // Find which protocol this position belongs to
        let protocol_id = {
            let user_positions = self.user_positions.read().await;
            
            if let Some(positions) = user_positions.get(&user_id) {
                if !positions.contains(&position_id) {
                    return Err(anyhow!("Position not found for user"));
                }
            } else {
                return Err(anyhow!("No positions found for user"));
            }
            
            // Get position details to find the protocol
            for protocol in self.registry.list_protocols().await? {
                let adapter = self.registry.get_adapter(&protocol.id).await?;
                if let Ok(position) = adapter.get_position_value(&position_id).await {
                    if position > Decimal::ZERO {
                        // Found the protocol
                        protocol.id
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            
            return Err(anyhow!("Protocol not found for position"));
        };
        
        let adapter = self.registry.get_adapter(&protocol_id).await?;
        adapter.close_position(&position_id).await
    }
    
    pub async fn get_user_positions(&self, user_id: &UserId) -> Result<Vec<Position>> {
        let user_positions = self.user_positions.read().await;
        
        let position_ids = match user_positions.get(user_id) {
            Some(ids) => ids.clone(),
            None => return Ok(Vec::new()),
        };
        
        let mut positions = Vec::new();
        
        for protocol in self.registry.list_protocols().await? {
            let adapter = self.registry.get_adapter(&protocol.id).await?;
            
            for position_id in &position_ids {
                match adapter.update_position(position_id).await {
                    Ok(position) => positions.push(position),
                    Err(_) => continue, // Position not found in this protocol
                }
            }
        }
        
        Ok(positions)
    }
    
    pub async fn harvest_rewards(
        &self,
        user_id: UserId,
        position_id: PositionId,
    ) -> Result<(Position, DeFiTransaction)> {
        // Find which protocol this position belongs to (similar to close_position)
        let protocol_id = {
            let user_positions = self.user_positions.read().await;
            
            if let Some(positions) = user_positions.get(&user_id) {
                if !positions.contains(&position_id) {
                    return Err(anyhow!("Position not found for user"));
                }
            } else {
                return Err(anyhow!("No positions found for user"));
            }
            
            // Get position details to find the protocol
            for protocol in self.registry.list_protocols().await? {
                let adapter = self.registry.get_adapter(&protocol.id).await?;
                if let Ok(position) = adapter.get_position_value(&position_id).await {
                    if position > Decimal::ZERO {
                        // Found the protocol
                        protocol.id
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            
            return Err(anyhow!("Protocol not found for position"));
        };
        
        let adapter = self.registry.get_adapter(&protocol_id).await?;
        adapter.harvest_rewards(&position_id).await
    }
    
    pub async fn get_portfolio_value(&self, user_id: &UserId) -> Result<Decimal> {
        let positions = self.get_user_positions(user_id).await?;
        
        let total_value = positions.iter().fold(Decimal::ZERO, |acc, position| {
            if position.status == PositionStatus::Active {
                acc + position.value_usd
            } else {
                acc
            }
        });
        
        Ok(total_value)
    }
    
    pub async fn check_positions_health(&self, user_id: &UserId) -> Result<Vec<(Position, Decimal)>> {
        let positions = self.get_user_positions(user_id).await?;
        let mut health_data = Vec::new();
        
        for position in positions {
            if position.status == PositionStatus::Active {
                for protocol in self.registry.list_protocols().await? {
                    let adapter = self.registry.get_adapter(&protocol.id).await?;
                    
                    if let Ok(health_factor) = adapter.check_position_health(&position.id).await {
                        health_data.push((position.clone(), health_factor));
                        break;
                    }
                }
            }
        }
        
        Ok(health_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[tokio::test]
    async fn test_uniswap_adapter() {
        // Create adapter and initialize pools
        let adapter = Arc::new(UniswapAdapter::new());
        adapter.initialize_pools().await.unwrap();
        
        // List pools
        let pools = adapter.list_pools().await.unwrap();
        assert_eq!(pools.len(), 3);
        
        // Create a position
        let user_id = Uuid::new_v4();
        let (position, transaction) = adapter.create_position(
            user_id,
            &"uniswap-v3-eth-usdt-500".to_string(),
            vec![
                AssetAmount {
                    asset_id: "ETH".to_string(),
                    amount: dec!(1),
                },
                AssetAmount {
                    asset_id: "USDT".to_string(),
                    amount: dec!(3000),
                },
            ],
        ).await.unwrap();
        
        // Verify position
        assert_eq!(position.user_id, user_id);
        assert_eq!(position.protocol_id, "uniswap-v3");
        assert_eq!(position.pool_id, Some("uniswap-v3-eth-usdt-500".to_string()));
        assert_eq!(position.status, PositionStatus::Active);
        
        // Get position value
        let value = adapter.get_position_value(&position.id).await.unwrap();
        assert!(value > Decimal::ZERO);
        
        // Harvest rewards
        let (updated_position, reward_tx) = adapter.harvest_rewards(&position.id).await.unwrap();
        assert_eq!(reward_tx.transaction_type, DeFiTransactionType::ClaimRewards);
        
        // Close position
        let (closed_position, close_tx) = adapter.close_position(&position.id).await.unwrap();
        assert_eq!(closed_position.status, PositionStatus::Closed);
        assert_eq!(close_tx.transaction_type, DeFiTransactionType::RemoveLiquidity);
    }
    
    #[tokio::test]
    async fn test_aave_adapter() {
        // Create adapter and initialize pools
        let adapter = Arc::new(AaveLendingAdapter::new());
        adapter.initialize_pools().await.unwrap();
        
        // List pools
        let pools = adapter.list_pools().await.unwrap();
        assert_eq!(pools.len(), 3);
        
        // Create a lending position
        let user_id = Uuid::new_v4();
        let (position, transaction) = adapter.create_position(
            user_id,
            &"aave-v3-eth-market".to_string(),
            vec![
                AssetAmount {
                    asset_id: "ETH".to_string(),
                    amount: dec!(2),
                },
            ],
        ).await.unwrap();
        
        // Verify position
        assert_eq!(position.user_id, user_id);
        assert_eq!(position.protocol_id, "aave-v3");
        assert_eq!(position.pool_id, Some("aave-v3-eth-market".to_string()));
        assert_eq!(position.status, PositionStatus::Active);
        
        // Get position value
        let value = adapter.get_position_value(&position.id).await.unwrap();
        assert!(value > Decimal::ZERO);
        
        // Create a borrowing position
        let (borrow_position, borrowing_position, borrow_tx) = adapter.create_borrow_position(
            user_id,
            &position.id,
            &"aave-v3-usdt-market".to_string(),
            dec!(1000),
        ).await.unwrap();
        
        // Verify borrowing position
        assert_eq!(borrow_position.user_id, user_id);
        assert_eq!(borrow_position.protocol_id, "aave-v3");
        assert_eq!(borrow_position.pool_id, Some("aave-v3-usdt-market".to_string()));
        assert_eq!(borrow_position.status, PositionStatus::Active);
        assert_eq!(borrowing_position.asset_borrowed, "USDT");
        assert_eq!(borrowing_position.amount_borrowed, dec!(1000));
        
        // Check position health
        let health = adapter.check_position_health(&borrow_position.id).await.unwrap();
        assert!(health > Decimal::ONE);
        
        // Close lending position (should fail due to active borrowing)
        let close_result = adapter.close_position(&position.id).await;
        assert!(close_result.is_err());
    }
    
    #[tokio::test]
    async fn test_defi_manager() {
        // Create registry and adapters
        let registry = Arc::new(ProtocolRegistry::new());
        let uniswap_adapter = Arc::new(UniswapAdapter::new());
        let aave_adapter = Arc::new(AaveLendingAdapter::new());
        
        // Initialize pools
        uniswap_adapter.initialize_pools().await.unwrap();
        aave_adapter.initialize_pools().await.unwrap();
        
        // Register adapters
        registry.register_adapter(uniswap_adapter.clone()).await.unwrap();
        registry.register_adapter(aave_adapter.clone()).await.unwrap();
        
        // Create DeFi manager
        let defi_manager = DeFiManager::new(registry.clone());
        
        // List protocols
        let protocols = defi_manager.list_protocols().await.unwrap();
        assert_eq!(protocols.len(), 2);
        
        // Create a user
        let user_id = Uuid::new_v4();
        
        // Create positions in different protocols
        let (uni_position, _) = defi_manager.create_position(
            user_id,
            &"uniswap-v3".to_string(),
            &"uniswap-v3-eth-usdt-500".to_string(),
            vec![
                AssetAmount {
                    asset_id: "ETH".to_string(),
                    amount: dec!(1),
                },
                AssetAmount {
                    asset_id: "USDT".to_string(),
                    amount: dec!(3000),
                },
            ],
        ).await.unwrap();
        
        let (aave_position, _) = defi_manager.create_position(
            user_id,
            &"aave-v3".to_string(),
            &"aave-v3-eth-market".to_string(),
            vec![
                AssetAmount {
                    asset_id: "ETH".to_string(),
                    amount: dec!(2),
                },
            ],
        ).await.unwrap();
        
        // Get user positions
        let positions = defi_manager.get_user_positions(&user_id).await.unwrap();
        assert_eq!(positions.len(), 2);
        
        // Get portfolio value
        let portfolio_value = defi_manager.get_portfolio_value(&user_id).await.unwrap();
        // Value should include both positions: 1 ETH + 3000 USDT from Uniswap, and 2 ETH from Aave
        // 1 ETH = $3,000, 3000 USDT = $3,000, 2 ETH = $6,000, Total = $12,000
        assert_eq!(portfolio_value, dec!(12000));
        
        // Close Uniswap position
        let (closed_position, _) = defi_manager.close_position(user_id, uni_position.id).await.unwrap();
        assert_eq!(closed_position.status, PositionStatus::Closed);
        
        // Get updated portfolio value
        let updated_value = defi_manager.get_portfolio_value(&user_id).await.unwrap();
        // Now only the Aave position should be active: 2 ETH = $6,000
        assert_eq!(updated_value, dec!(6000));
    }
}
// src/defi/mod.rs

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

/// Type definitions for DeFi integrations
pub type ProtocolId = String;
pub type PoolId = String;
pub type AssetId = String;
pub type UserId = Uuid;
pub type PositionId = Uuid;
pub type TransactionId = Uuid;

/// Protocol types supported by the DeFi integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolType {
    LiquidityPool,       // AMM liquidity pools like Uniswap
    LendingProtocol,     // Lending/borrowing like Aave or Compound
    StakingProtocol,     // Staking like Lido
    YieldFarm,           // Yield farming
    SyntheticAsset,      // Synthetic assets like Synthetix
    DerivativeProtocol,  // Derivatives protocols
}

/// Status of a position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStatus {
    Active,
    Closing,
    Closed,
    Failed,
}

/// Risk level of a protocol or pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Representation of a DeFi protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub id: ProtocolId,
    pub name: String,
    pub protocol_type: ProtocolType,
    pub chain_id: String,
    pub contracts: Vec<ContractInfo>,
    pub description: String,
    pub risk_level: RiskLevel,
    pub tvl: Decimal,           // Total Value Locked in USD
    pub apy_range: (Decimal, Decimal),  // Min and max APY
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Smart contract information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub name: String,
    pub address: String,
    pub abi: String,
}

/// Pool or vault within a protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub id: PoolId,
    pub protocol_id: ProtocolId,
    pub name: String,
    pub assets: Vec<AssetId>,
    pub tvl: Decimal,
    pub apy: Decimal,
    pub fees: PoolFees,
    pub is_active: bool,
    pub risk_level: RiskLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fee structure for a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolFees {
    pub entry_fee: Decimal,     // Fee when entering the pool
    pub exit_fee: Decimal,      // Fee when exiting the pool
    pub performance_fee: Decimal, // Fee on profits
    pub management_fee: Decimal,  // Annual management fee
}

/// User position in a DeFi protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: PositionId,
    pub user_id: UserId,
    pub protocol_id: ProtocolId,
    pub pool_id: Option<PoolId>,
    pub assets_deposited: Vec<AssetAmount>,
    pub tokens_received: Vec<AssetAmount>,
    pub value_usd: Decimal,
    pub apy: Decimal,
    pub status: PositionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Asset with amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAmount {
    pub asset_id: AssetId,
    pub amount: Decimal,
}

/// Transaction in a DeFi protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiTransaction {
    pub id: TransactionId,
    pub user_id: UserId,
    pub protocol_id: ProtocolId,
    pub pool_id: Option<PoolId>,
    pub position_id: Option<PositionId>,
    pub transaction_type: DeFiTransactionType,
    pub assets_in: Vec<AssetAmount>,
    pub assets_out: Vec<AssetAmount>,
    pub value_usd: Decimal,
    pub blockchain_txid: Option<String>,
    pub status: DeFiTransactionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Type of DeFi transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeFiTransactionType {
    Deposit,
    Withdraw,
    Stake,
    Unstake,
    Claim,
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Borrow,
    Repay,
