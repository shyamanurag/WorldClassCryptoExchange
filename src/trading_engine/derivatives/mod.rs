use std::sync::Arc;
use tokio::sync::RwLock;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::trading_engine::matching_engine::{Side, OrderStatus, OrderType};

/// Type definitions for derivatives trading
pub type ContractId = String;
pub type PositionId = Uuid;
pub type UserId = Uuid;
pub type FundingRate = Decimal;

/// Contract types for derivatives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractType {
    /// Futures contract with a fixed expiry date
    Futures,
    /// Perpetual contract with no expiry date
    Perpetual,
    /// Options contract (call or put)
    Option,
}

/// Option types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

/// Margin types for leveraged trading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginType {
    /// Isolated margin - risk limited to position
    Isolated,
    /// Cross margin - shared across all positions
    Cross,
}

/// Position direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionDirection {
    Long,
    Short,
}

/// Status of a position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
    Liquidated,
}

/// Representation of a derivatives contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: ContractId,
    pub base_asset: String,
    pub quote_asset: String,
    pub contract_type: ContractType,
    pub tick_size: Decimal,
    pub lot_size: Decimal,
    pub leverage_max: Decimal,
    pub maintenance_margin_ratio: Decimal,
    pub liquidation_fee_ratio: Decimal,
    pub maker_fee_rate: Decimal,
    pub taker_fee_rate: Decimal,
    pub expiry_time: Option<DateTime<Utc>>,
    pub settlement_asset: String,
    pub option_type: Option<OptionType>,
    pub strike_price: Option<Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Representation of a trader's position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: PositionId,
    pub user_id: UserId,
    pub contract_id: ContractId,
    pub direction: PositionDirection,
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub leverage: Decimal,
    pub liquidation_price: Decimal,
    pub margin_type: MarginType,
    pub margin_amount: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub status: PositionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl Position {
    pub fn new(
        user_id: UserId,
        contract_id: ContractId,
        direction: PositionDirection,
        quantity: Decimal,
        entry_price: Decimal,
        leverage: Decimal,
        margin_type: MarginType,
        margin_amount: Decimal,
    ) -> Self {
        let now = Utc::now();
        
        // In a real implementation, this would be calculated based on entry price,
        // leverage, and maintenance margin ratio
        let liquidation_price = if direction == PositionDirection::Long {
            entry_price * Decimal::new(95, 2) // 95% of entry price for long
        } else {
            entry_price * Decimal::new(105, 2) // 105% of entry price for short
        };
        
        Position {
            id: Uuid::new_v4(),
            user_id,
            contract_id,
            direction,
            quantity,
            entry_price,
            leverage,
            liquidation_price,
            margin_type,
            margin_amount,
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            status: PositionStatus::Open,
            created_at: now,
            updated_at: now,
            closed_at: None,
        }
    }
    
    pub fn update_unrealized_pnl(&mut self, mark_price: Decimal) -> Decimal {
        let price_diff = match self.direction {
            PositionDirection::Long => mark_price - self.entry_price,
            PositionDirection::Short => self.entry_price - mark_price,
        };
        
        self.unrealized_pnl = price_diff * self.quantity;
        self.updated_at = Utc::now();
        
        self.unrealized_pnl
    }
    
    pub fn close(&mut self, exit_price: Decimal, exit_quantity: Decimal) -> Result<Decimal> {
        if exit_quantity > self.quantity {
            return Err(anyhow::anyhow!("Exit quantity exceeds position quantity"));
        }
        
        let price_diff = match self.direction {
            PositionDirection::Long => exit_price - self.entry_price,
            PositionDirection::Short => self.entry_price - exit_price,
        };
        
        let realized_pnl_for_exit = price_diff * exit_quantity;
        self.realized_pnl += realized_pnl_for_exit;
        self.quantity -= exit_quantity;
        
        if self.quantity == Decimal::ZERO {
            self.status = PositionStatus::Closed;
            self.closed_at = Some(Utc::now());
        }
        
        self.updated_at = Utc::now();
        
        Ok(realized_pnl_for_exit)
    }
    
    pub fn liquidate(&mut self, liquidation_price: Decimal) -> Result<Decimal> {
        if self.status != PositionStatus::Open {
            return Err(anyhow::anyhow!("Cannot liquidate a non-open position"));
        }
        
        let price_diff = match self.direction {
            PositionDirection::Long => liquidation_price - self.entry_price,
            PositionDirection::Short => self.entry_price - liquidation_price,
        };
        
        let realized_pnl = price_diff * self.quantity;
        self.realized_pnl += realized_pnl;
        self.status = PositionStatus::Liquidated;
        self.closed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        
        Ok(realized_pnl)
    }
    
    pub fn check_liquidation(&self, mark_price: Decimal, maintenance_margin_ratio: Decimal) -> bool {
        match self.direction {
            PositionDirection::Long => mark_price <= self.liquidation_price,
            PositionDirection::Short => mark_price >= self.liquidation_price,
        }
    }
}

/// Funding payment for perpetual contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingPayment {
    pub id: Uuid,
    pub user_id: UserId,
    pub contract_id: ContractId,
    pub position_id: PositionId,
    pub funding_rate: FundingRate,
    pub payment_amount: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// Manager for derivatives contracts
pub struct ContractManager {
    contracts: RwLock<Vec<Contract>>,
}

impl ContractManager {
    pub fn new() -> Self {
        ContractManager {
            contracts: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_contract(&self, contract: Contract) -> Result<()> {
        let mut contracts = self.contracts.write().await;
        
        // Check if contract already exists
        if contracts.iter().any(|c| c.id == contract.id) {
            return Err(anyhow::anyhow!("Contract already exists"));
        }
        
        contracts.push(contract);
        Ok(())
    }
    
    pub async fn get_contract(&self, contract_id: &str) -> Option<Contract> {
        let contracts = self.contracts.read().await;
        contracts.iter().find(|c| c.id == contract_id).cloned()
    }
    
    pub async fn list_active_contracts(&self) -> Vec<Contract> {
        let contracts = self.contracts.read().await;
        contracts.iter().filter(|c| c.is_active).cloned().collect()
    }
    
    pub async fn update_contract(&self, contract: Contract) -> Result<()> {
        let mut contracts = self.contracts.write().await;
        
        if let Some(index) = contracts.iter().position(|c| c.id == contract.id) {
            contracts[index] = contract;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Contract not found"))
        }
    }
}

/// Manager for trader positions
pub struct PositionManager {
    positions: RwLock<Vec<Position>>,
}

impl PositionManager {
    pub fn new() -> Self {
        PositionManager {
            positions: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_position(&self, position: Position) -> Result<PositionId> {
        let mut positions = self.positions.write().await;
        let position_id = position.id;
        positions.push(position);
        Ok(position_id)
    }
    
    pub async fn get_position(&self, position_id: &PositionId) -> Option<Position> {
        let positions = self.positions.read().await;
        positions.iter().find(|p| p.id == *position_id).cloned()
    }
    
    pub async fn get_user_positions(&self, user_id: &UserId) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.iter().filter(|p| p.user_id == *user_id).cloned().collect()
    }
    
    pub async fn get_user_open_positions(&self, user_id: &UserId) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.iter()
            .filter(|p| p.user_id == *user_id && p.status == PositionStatus::Open)
            .cloned()
            .collect()
    }
    
    pub async fn update_position(&self, position: Position) -> Result<()> {
        let mut positions = self.positions.write().await;
        
        if let Some(index) = positions.iter().position(|p| p.id == position.id) {
            positions[index] = position;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Position not found"))
        }
    }
    
    pub async fn update_unrealized_pnl(&self, position_id: &PositionId, mark_price: Decimal) -> Result<Decimal> {
        let mut positions = self.positions.write().await;
        
        if let Some(position) = positions.iter_mut().find(|p| p.id == *position_id) {
            let pnl = position.update_unrealized_pnl(mark_price);
            Ok(pnl)
        } else {
            Err(anyhow::anyhow!("Position not found"))
        }
    }
    
    pub async fn close_position(
        &self,
        position_id: &PositionId,
        exit_price: Decimal,
        exit_quantity: Decimal,
    ) -> Result<Decimal> {
        let mut positions = self.positions.write().await;
        
        if let Some(position) = positions.iter_mut().find(|p| p.id == *position_id) {
            position.close(exit_price, exit_quantity)
        } else {
            Err(anyhow::anyhow!("Position not found"))
        }
    }
    
    pub async fn liquidate_position(&self, position_id: &PositionId, liquidation_price: Decimal) -> Result<Decimal> {
        let mut positions = self.positions.write().await;
        
        if let Some(position) = positions.iter_mut().find(|p| p.id == *position_id) {
            position.liquidate(liquidation_price)
        } else {
            Err(anyhow::anyhow!("Position not found"))
        }
    }
    
    pub async fn check_liquidations(
        &self,
        contract_id: &ContractId,
        mark_price: Decimal,
        maintenance_margin_ratio: Decimal,
    ) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.iter()
            .filter(|p| p.contract_id == *contract_id && p.status == PositionStatus::Open)
            .filter(|p| p.check_liquidation(mark_price, maintenance_margin_ratio))
            .cloned()
            .collect()
    }
}

/// Funding rate calculator for perpetual contracts
pub struct FundingRateCalculator {
    // Funding rate parameters
    interest_rate: Decimal,
    premium_index_weight: Decimal,
    funding_interval_hours: u32,
}

impl FundingRateCalculator {
    pub fn new(interest_rate: Decimal, premium_index_weight: Decimal, funding_interval_hours: u32) -> Self {
        FundingRateCalculator {
            interest_rate,
            premium_index_weight,
            funding_interval_hours,
        }
    }
    
    pub fn calculate_funding_rate(
        &self,
        mark_price: Decimal,
        index_price: Decimal,
        long_positions_size: Decimal,
        short_positions_size: Decimal,
    ) -> FundingRate {
        // Premium index component (mark price vs index price)
        let premium_index = if index_price > Decimal::ZERO {
            (mark_price - index_price) / index_price
        } else {
            Decimal::ZERO
        };
        
        // Interest rate component (fixed)
        let interest_component = self.interest_rate / Decimal::from(24) * Decimal::from(self.funding_interval_hours);
        
        // Premium index component (weighted)
        let premium_component = premium_index * self.premium_index_weight;
        
        // Final funding rate
        interest_component + premium_component
    }
    
    pub fn calculate_funding_payment(&self, position: &Position, funding_rate: FundingRate) -> Decimal {
        let notional_value = position.quantity * position.entry_price;
        
        // For long positions, positive funding rate means payment, negative means receipt
        // For short positions, it's the opposite
        match position.direction {
            PositionDirection::Long => notional_value * funding_rate * Decimal::from(-1),
            PositionDirection::Short => notional_value * funding_rate,
        }
    }
}

/// Derivatives engine for managing leveraged trading
pub struct DerivativesEngine {
    contract_manager: Arc<ContractManager>,
    position_manager: Arc<PositionManager>,
    funding_calculator: FundingRateCalculator,
}

impl DerivativesEngine {
    pub fn new(
        contract_manager: Arc<ContractManager>,
        position_manager: Arc<PositionManager>,
        funding_calculator: FundingRateCalculator,
    ) -> Self {
        DerivativesEngine {
            contract_manager,
            position_manager,
            funding_calculator,
        }
    }
    
    pub async fn open_position(
        &self,
        user_id: UserId,
        contract_id: &str,
        direction: PositionDirection,
        quantity: Decimal,
        leverage: Decimal,
        margin_type: MarginType,
    ) -> Result<Position> {
        // Get contract
        let contract = match self.contract_manager.get_contract(contract_id).await {
            Some(contract) => contract,
            None => return Err(anyhow::anyhow!("Contract not found")),
        };
        
        // Validate leverage
        if leverage > contract.leverage_max {
            return Err(anyhow::anyhow!("Leverage exceeds maximum allowed"));
        }
        
        // Get current mark price (in a real implementation, this would come from a price oracle)
        let mark_price = Decimal::new(5000000, 2); // Example: $50,000.00
        
        // Calculate required margin
        let notional_value = quantity * mark_price;
        let margin_amount = notional_value / leverage;
        
        // Create a new position
        let position = Position::new(
            user_id,
            contract_id.to_string(),
            direction,
            quantity,
            mark_price,
            leverage,
            margin_type,
            margin_amount,
        );
        
        // Add position
        let position_id = self.position_manager.add_position(position.clone()).await?;
        
        Ok(position)
    }
    
    pub async fn close_position(
        &self,
        user_id: UserId,
        position_id: PositionId,
        quantity: Option<Decimal>,
    ) -> Result<Decimal> {
        // Get position
        let position = match self.position_manager.get_position(&position_id).await {
            Some(position) => position,
            None => return Err(anyhow::anyhow!("Position not found")),
        };
        
        // Verify user owns the position
        if position.user_id != user_id {
            return Err(anyhow::anyhow!("Unauthorized"));
        }
        
        // Check if position is already closed
        if position.status != PositionStatus::Open {
            return Err(anyhow::anyhow!("Position is not open"));
        }
        
        // Determine quantity to close
        let close_quantity = quantity.unwrap_or(position.quantity);
        
        // Get current mark price (in a real implementation, this would come from a price oracle)
        let mark_price = Decimal::new(5200000, 2); // Example: $52,000.00
        
        // Close the position
        let realized_pnl = self.position_manager.close_position(&position_id, mark_price, close_quantity).await?;
        
        Ok(realized_pnl)
    }
    
    pub async fn update_positions_pnl(&self, contract_id: &str) -> Result<()> {
        // Get contract
        let contract = match self.contract_manager.get_contract(contract_id).await {
            Some(contract) => contract,
            None => return Err(anyhow::anyhow!("Contract not found")),
        };
        
        // Get current mark price (in a real implementation, this would come from a price oracle)
        let mark_price = Decimal::new(5200000, 2); // Example: $52,000.00
        
        // Get all open positions for this contract
        let positions = self.position_manager.get_user_open_positions(&UserId::nil()).await;
        
        // Update unrealized PnL for each position
        for position in positions {
            if position.contract_id == contract_id {
                let _ = self.position_manager.update_unrealized_pnl(&position.id, mark_price).await;
            }
        }
        
        Ok(())
    }
    
    pub async fn check_liquidations(&self, contract_id: &str) -> Result<Vec<Position>> {
        // Get contract
        let contract = match self.contract_manager.get_contract(contract_id).await {
            Some(contract) => contract,
            None => return Err(anyhow::anyhow!("Contract not found")),
        };
        
        // Get current mark price (in a real implementation, this would come from a price oracle)
        let mark_price = Decimal::new(5200000, 2); // Example: $52,000.00
        
        // Check for positions that need to be liquidated
        let liquidation_candidates = self.position_manager
            .check_liquidations(&contract.id, mark_price, contract.maintenance_margin_ratio)
            .await;
        
        // Liquidate positions
        let mut liquidated_positions = Vec::new();
        for position in liquidation_candidates {
            match self.position_manager.liquidate_position(&position.id, mark_price).await {
                Ok(_) => {
                    liquidated_positions.push(position);
                }
                Err(e) => {
                    eprintln!("Failed to liquidate position {}: {}", position.id, e);
                }
            }
        }
        
        Ok(liquidated_positions)
    }
    
    pub async fn calculate_funding_payments(&self, contract_id: &str) -> Result<Vec<FundingPayment>> {
        // Get contract
        let contract = match self.contract_manager.get_contract(contract_id).await {
            Some(contract) => contract,
            None => return Err(anyhow::anyhow!("Contract not found")),
        };
        
        // Only perpetual contracts have funding
        if contract.contract_type != ContractType::Perpetual {
            return Ok(Vec::new());
        }
        
        // Get current mark price and index price
        let mark_price = Decimal::new(5200000, 2); // Example: $52,000.00
        let index_price = Decimal::new(5190000, 2); // Example: $51,900.00
        
        // Get all open positions for this contract
        let positions = self.position_manager
            .positions.read().await
            .iter()
            .filter(|p| p.contract_id == contract_id && p.status == PositionStatus::Open)
            .cloned()
            .collect::<Vec<_>>();
        
        // Calculate total long and short positions
        let (long_positions_size, short_positions_size) = positions.iter().fold(
            (Decimal::ZERO, Decimal::ZERO),
            |(long_sum, short_sum), position| {
                match position.direction {
                    PositionDirection::Long => (long_sum + position.quantity, short_sum),
                    PositionDirection::Short => (long_sum, short_sum + position.quantity),
                }
            },
        );
        
        // Calculate funding rate
        let funding_rate = self.funding_calculator.calculate_funding_rate(
            mark_price,
            index_price,
            long_positions_size,
            short_positions_size,
        );
        
        // Calculate funding payments for each position
        let mut funding_payments = Vec::new();
        for position in positions {
            let payment_amount = self.funding_calculator.calculate_funding_payment(&position, funding_rate);
            
            if payment_amount != Decimal::ZERO {
                let funding_payment = FundingPayment {
                    id: Uuid::new_v4(),
                    user_id: position.user_id,
                    contract_id: position.contract_id.clone(),
                    position_id: position.id,
                    funding_rate,
                    payment_amount,
                    timestamp: Utc::now(),
                };
                
                funding_payments.push(funding_payment);
            }
        }
        
        Ok(funding_payments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[tokio::test]
    async fn test_open_and_close_position() {
        // Create managers
        let contract_manager = Arc::new(ContractManager::new());
        let position_manager = Arc::new(PositionManager::new());
        
        // Create funding calculator
        let funding_calculator = FundingRateCalculator::new(
            dec!(0.0001), // 0.01% interest rate
            dec!(0.0005), // 0.05% premium index weight
            8, // 8-hour funding interval
        );
        
        // Create derivatives engine
        let derivatives_engine = DerivativesEngine::new(
            contract_manager.clone(),
            position_manager.clone(),
            funding_calculator,
        );
        
        // Create a contract
        let contract = Contract {
            id: "BTC-PERP".to_string(),
            base_asset: "BTC".to_string(),
            quote_asset: "USDT".to_string(),
            contract_type: ContractType::Perpetual,
            tick_size: dec!(0.5),
            lot_size: dec!(0.001),
            leverage_max: dec!(100),
            maintenance_margin_ratio: dec!(0.01),
            liquidation_fee_ratio: dec!(0.005),
            maker_fee_rate: dec!(0.0002),
            taker_fee_rate: dec!(0.0005),
            expiry_time: None,
            settlement_asset: "USDT".to_string(),
            option_type: None,
            strike_price: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Add contract
        contract_manager.add_contract(contract).await.unwrap();
        
        // Open a position
        let user_id = Uuid::new_v4();
        let position = derivatives_engine.open_position(
            user_id,
            "BTC-PERP",
            PositionDirection::Long,
            dec!(0.1), // 0.1 BTC
            dec!(10),  // 10x leverage
            MarginType::Isolated,
        ).await.unwrap();
        
        // Verify position
        assert_eq!(position.direction, PositionDirection::Long);
        assert_eq!(position.quantity, dec!(0.1));
        assert_eq!(position.leverage, dec!(10));
        assert_eq!(position.status, PositionStatus::Open);
        
        // Close the position
        let realized_pnl = derivatives_engine.close_position(
            user_id,
            position.id,
            Some(dec!(0.05)), // Close half of the position
        ).await.unwrap();
        
        // Verify PnL (should be positive as the price increased)
        assert!(realized_pnl > Decimal::ZERO);
        
        // Verify position update
        let updated_position = position_manager.get_position(&position.id).await.unwrap();
        assert_eq!(updated_position.quantity, dec!(0.05)); // Half of the original quantity
        assert_eq!(updated_position.status, PositionStatus::Open); // Still open
        
        // Close the rest of the position
        let realized_pnl = derivatives_engine.close_position(
            user_id,
            position.id,
            None, // Close the entire remaining position
        ).await.unwrap();
        
        // Verify PnL
        assert!(realized_pnl > Decimal::ZERO);
        
        // Verify position closure
        let closed_position = position_manager.get_position(&position.id).await.unwrap();
        assert_eq!(closed_position.quantity, dec!(0)); // No quantity left
        assert_eq!(closed_position.status, PositionStatus::Closed); // Now closed
        assert!(closed_position.closed_at.is_some()); // Has a closure timestamp
    }
}

// src/trading_engine/margin/mod.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::trading_engine::matching_engine::{Side, OrderStatus, OrderType};

/// Type definitions for margin trading
pub type MarginAccountId = Uuid;
pub type UserId = Uuid;
pub type AssetId = String;
pub type MarginCallId = Uuid;

/// Status of a margin account
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginAccountStatus {
    Active,
    MarginCall,
    Liquidating,
    Closed,
}

/// Type of margin call
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginCallType {
    Warning,          // Initial warning
    MaintenanceCall,  // Margin below maintenance requirement
    LiquidationCall,  // Margin below liquidation threshold
}

/// Representation of a margin account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginAccount {
    pub id: MarginAccountId,
    pub user_id: UserId,
    pub status: MarginAccountStatus,
    pub is_isolated: bool,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub base_balance: Decimal,
    pub quote_balance: Decimal,
    pub borrowed_base: Decimal,
    pub borrowed_quote: Decimal,
    pub interest_rate_base: Decimal,
    pub interest_rate_quote: Decimal,
    pub last_interest_time: DateTime<Utc>,
    pub margin_level: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MarginAccount {
    pub fn new(
        user_id: UserId,
        is_isolated: bool,
        base_asset: AssetId,
        quote_asset: AssetId,
        initial_base: Decimal,
        initial_quote: Decimal,
        interest_rate_base: Decimal,
        interest_rate_quote: Decimal,
    ) -> Self {
        let now = Utc::now();
        
        MarginAccount {
            id: Uuid::new_v4(),
            user_id,
            status: MarginAccountStatus::Active,
            is_isolated,
            base_asset,
            quote_asset,
            base_balance: initial_base,
            quote_balance: initial_quote,
            borrowed_base: Decimal::ZERO,
            borrowed_quote: Decimal::ZERO,
            interest_rate_base,
            interest_rate_quote,
            last_interest_time: now,
            margin_level: Decimal::ONE, // Will be recalculated
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Calculate the current equity in the account
    pub fn calculate_equity(&self, base_price: Decimal) -> Decimal {
        // Convert base asset to quote asset equivalent
        let base_value = self.base_balance * base_price;
        
        // Add quote balance to get total equity
        base_value + self.quote_balance
    }
    
    /// Calculate total borrowed value
    pub fn calculate_borrowed_value(&self, base_price: Decimal) -> Decimal {
        // Convert borrowed base to quote equivalent
        let borrowed_base_value = self.borrowed_base * base_price;
        
        // Add borrowed quote
        borrowed_base_value + self.borrowed_quote
    }
    
    /// Calculate the current margin level
    pub fn calculate_margin_level(&self, base_price: Decimal) -> Decimal {
        let equity = self.calculate_equity(base_price);
        let borrowed = self.calculate_borrowed_value(base_price);
        
        if borrowed > Decimal::ZERO {
            equity / borrowed
        } else {
            // If nothing is borrowed, margin level is effectively infinite
            // For practical purposes, use a high value
            Decimal::from(100)
        }
    }
    
    /// Update the margin level based on current prices
    pub fn update_margin_level(&mut self, base_price: Decimal) -> Decimal {
        self.margin_level = self.calculate_margin_level(base_price);
        self.updated_at = Utc::now();
        
        self.margin_level
    }
    
    /// Borrow additional assets
    pub fn borrow(&mut self, asset_is_base: bool, amount: Decimal) -> Result<()> {
        if amount <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Borrow amount must be positive"));
        }
        
        if asset_is_base {
            self.borrowed_base += amount;
            self.base_balance += amount;
        } else {
            self.borrowed_quote += amount;
            self.quote_balance += amount;
        }
        
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Repay borrowed assets
    pub fn repay(&mut self, asset_is_base: bool, amount: Decimal) -> Result<()> {
        if amount <= Decimal::ZERO {
            return Err(anyhow::anyhow!("Repay amount must be positive"));
        }
        
        if asset_is_base {
            if amount > self.borrowed_base {
                return Err(anyhow::anyhow!("Repay amount exceeds borrowed amount"));
            }
            
            if amount > self.base_balance {
                return Err(anyhow::anyhow!("Insufficient balance for repayment"));
            }
            
            self.borrowed_base -= amount;
            self.base_balance -= amount;
        } else {
            if amount > self.borrowed_quote {
                return Err(anyhow::anyhow!("Repay amount exceeds borrowed amount"));
            }
            
            if amount > self.quote_balance {
                return Err(anyhow::anyhow!("Insufficient balance for repayment"));
            }
            
            self.borrowed_quote -= amount;
            self.quote_balance -= amount;
        }
        
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Apply interest to borrowed assets
    pub fn apply_interest(&mut self) -> (Decimal, Decimal) {
        let now = Utc::now();
        let seconds_elapsed = (now - self.last_interest_time).num_seconds() as u64;
        
        // Convert annual interest rate to per-second rate
        // Formula: (1 + annual_rate)^(seconds/seconds_in_year) - 1
        let seconds_in_year = 31_536_000u64; // 365 days
        
        let base_interest_factor = ((Decimal::ONE + self.interest_rate_base).powf(
            Decimal::from(seconds_elapsed) / Decimal::from(seconds_in_year)
        )) - Decimal::ONE;
        
        let quote_interest_factor = ((Decimal::ONE + self.interest_rate_quote).powf(
            Decimal::from(seconds_elapsed) / Decimal::from(seconds_in_year)
        )) - Decimal::ONE;
        
        // Calculate interest amounts
        let base_interest = self.borrowed_base * base_interest_factor;
        let quote_interest = self.borrowed_quote * quote_interest_factor;
        
        // Apply interest
        self.borrowed_base += base_interest;
        self.borrowed_quote += quote_interest;
        
        // Update last interest time
        self.last_interest_time = now;
        self.updated_at = now;
        
        (base_interest, quote_interest)
    }
}

/// Representation of a margin call event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginCall {
    pub id: MarginCallId,
    pub account_id: MarginAccountId,
    pub call_type: MarginCallType,
    pub margin_level: Decimal,
    pub required_level: Decimal,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Manager for margin accounts
pub struct MarginAccountManager {
    accounts: RwLock<Vec<MarginAccount>>,
    margin_calls: RwLock<Vec<MarginCall>>,
    maintenance_margin_level: Decimal,
    liquidation_margin_level: Decimal,
}

impl MarginAccountManager {
    pub fn new(maintenance_margin_level: Decimal, liquidation_margin_level: Decimal) -> Self {
        MarginAccountManager {
            accounts: RwLock::new(Vec::new()),
            margin_calls: RwLock::new(Vec::new()),
            maintenance_margin_level,
            liquidation_margin_level,
        }
    }
    
    pub async fn create_account(
        &self,
        user_id: UserId,
        is_isolated: bool,
        base_asset: AssetId,
        quote_asset: AssetId,
        initial_base: Decimal,
        initial_quote: Decimal,
        interest_rate_base: Decimal,
        interest_rate_quote: Decimal,
    ) -> Result<MarginAccount> {
        let account = MarginAccount::new(
            user_id,
            is_isolated,
            base_asset,
            quote_asset,
            initial_base,
            initial_quote,
            interest_rate_base,
            interest_rate_quote,
        );
        
        let mut accounts = self.accounts.write().await;
        accounts.push(account.clone());
        
        Ok(account)
    }
    
    pub async fn get_account(&self, account_id: &MarginAccountId) -> Option<MarginAccount> {
        let accounts = self.accounts.read().await;
        accounts.iter().find(|a| a.id == *account_id).cloned()
    }
    
    pub async fn get_user_accounts(&self, user_id: &UserId) -> Vec<MarginAccount> {
        let accounts = self.accounts.read().await;
        accounts.iter().filter(|a| a.user_id == *user_id).cloned().collect()
    }
    
    pub async fn update_account(&self, account: MarginAccount) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        
        if let Some(index) = accounts.iter().position(|a| a.id == account.id) {
            accounts[index] = account;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }
    
    pub async fn borrow(&self, account_id: &MarginAccountId, asset_is_base: bool, amount: Decimal) -> Result<MarginAccount> {
        let mut accounts = self.accounts.write().await;
        
        if let Some(account) = accounts.iter_mut().find(|a| a.id == *account_id) {
            account.borrow(asset_is_base, amount)?;
            Ok(account.clone())
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }
    
    pub async fn repay(&self, account_id: &MarginAccountId, asset_is_base: bool, amount: Decimal) -> Result<MarginAccount> {
        let mut accounts = self.accounts.write().await;
        
        if let Some(account) = accounts.iter_mut().find(|a| a.id == *account_id) {
            account.repay(asset_is_base, amount)?;
            Ok(account.clone())
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }
    
    pub async fn apply_interest(&self, account_id: &MarginAccountId) -> Result<(Decimal, Decimal)> {
        let mut accounts = self.accounts.write().await;
        
        if let Some(account) = accounts.iter_mut().find(|a| a.id == *account_id) {
            Ok(account.apply_interest())
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }
    
    pub async fn update_margin_level(&self, account_id: &MarginAccountId, base_price: Decimal) -> Result<Decimal> {
        let mut accounts = self.accounts.write().await;
        
        if let Some(account) = accounts.iter_mut().find(|a| a.id == *account_id) {
            Ok(account.update_margin_level(base_price))
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }
    
    pub async fn check_margin_calls(&self, base_price: Decimal) -> Result<Vec<MarginCall>> {
        let mut accounts = self.accounts.write().await;
        let mut margin_calls = self.margin_calls.write().await;
        let mut new_calls = Vec::new();
        
        for account in accounts.iter_mut() {
            // Skip accounts with no borrowed assets
            if account.borrowed_base == Decimal::ZERO && account.borrowed_quote == Decimal::ZERO {
                continue;
            }
            
            // Update margin level
            let margin_level = account.update_margin_level(base_price);
            
            // Check for liquidation
            if margin_level <= self.liquidation_margin_level {
                account.status = MarginAccountStatus::Liquidating;
                
                let call = MarginCall {
                    id: Uuid::new_v4(),
                    account_id: account.id,
                    call_type: MarginCallType::LiquidationCall,
                    margin_level,
                    required_level: self.liquidation_margin_level,
                    timestamp: Utc::now(),
                    resolved: false,
                    resolved_at: None,
                };
                
                margin_calls.push(call.clone());
                new_calls.push(call);
            }
            // Check for maintenance margin call
            else if margin_level <= self.maintenance_margin_level {
                if account.status == MarginAccountStatus::Active {
                    account.status = MarginAccountStatus::MarginCall;
                }
                
                let call = MarginCall {
                    id: Uuid::new_v4(),
                    account_id: account.id,
                    call_type: MarginCallType::MaintenanceCall,
                    margin_level,
                    required_level: self.maintenance_margin_level,
                    timestamp: Utc::now(),
                    resolved: false,
                    resolved_at: None,
                };
                
                margin_calls.push(call.clone());
                new_calls.push(call);
            }
            // Check for warning (margin approaching maintenance level)
            else if margin_level <= self.maintenance_margin_level * Decimal::new(12, 1) { // 1.2x maintenance
                let call = MarginCall {
                    id: Uuid::new_v4(),
                    account_id: account.id,
                    call_type: MarginCallType::Warning,
                    margin_level,
                    required_level: self.maintenance_margin_level,
                    timestamp: Utc::now(),
                    resolved: false,
                    resolved_at: None,
                };
                
                margin_calls.push(call.clone());
                new_calls.push(call);
            }
            // Account is healthy, reset status if it was in margin call
            else if account.status == MarginAccountStatus::MarginCall {
                account.status = MarginAccountStatus::Active;
                
                // Resolve any outstanding maintenance margin calls
                for call in margin_calls.iter_mut() {
                    if call.account_id == account.id && !call.resolved &&
                        call.call_type != MarginCallType::LiquidationCall {
                        call.resolved = true;
                        call.resolved_at = Some(Utc::now());
                    }
                }
            }
        }
        
        Ok(new_calls)
    }
    
    pub async fn liquidate_account(&self, account_id: &MarginAccountId) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        let mut margin_calls = self.margin_calls.write().await;
        
        let account = accounts.iter_mut().find(|a| a.id == *account_id)
            .ok_or_else(|| anyhow::anyhow!("Account not found"))?;
        
        // Only liquidate accounts that are in liquidation status
        if account.status != MarginAccountStatus::Liquidating {
            return Err(anyhow::anyhow!("Account is not in liquidation status"));
        }
        
        // In a real implementation, we would sell the assets to repay the loans
        // Here we just reset the account balances and borrowed amounts
        
        // Reset account
        account.base_balance = Decimal::ZERO;
        account.quote_balance = Decimal::ZERO;
        account.borrowed_base = Decimal::ZERO;
        account.borrowed_quote = Decimal::ZERO;
        account.status = MarginAccountStatus::Closed;
        account.updated_at = Utc::now();
        
        // Resolve any outstanding margin calls
        for call in margin_calls.iter_mut() {
            if call.account_id == *account_id && !call.resolved {
                call.resolved = true;
                call.resolved_at = Some(Utc::now());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[tokio::test]
    async fn test_margin_account_creation_and_borrowing() {
        // Create margin account manager
        let margin_manager = MarginAccountManager::new(
            dec!(1.5),  // 150% maintenance margin requirement
            dec!(1.1),  // 110% liquidation threshold
        );
        
        // Create account
        let user_id = Uuid::new_v4();
        let account = margin_manager.create_account(
            user_id,
            true, // isolated margin
            "BTC".to_string(),
            "USDT".to_string(),
            dec!(0),   // 0 BTC initial
            dec!(1000), // 1000 USDT initial
            dec!(0.05), // 5% annual interest on BTC
            dec!(0.03), // 3% annual interest on USDT
        ).await.unwrap();
        
        // Verify account
        assert_eq!(account.user_id, user_id);
        assert_eq!(account.is_isolated, true);
        assert_eq!(account.base_asset, "BTC");
        assert_eq!(account.quote_asset, "USDT");
        assert_eq!(account.quote_balance, dec!(1000));
        assert_eq!(account.status, MarginAccountStatus::Active);
        
        // Borrow BTC
        let btc_price = dec!(50000); // $50,000 per BTC
        let updated_account = margin_manager.borrow(&account.id, true, dec!(0.1)).await.unwrap(); // Borrow 0.1 BTC
        
        // Verify borrow
        assert_eq!(updated_account.borrowed_base, dec!(0.1));
        assert_eq!(updated_account.base_balance, dec!(0.1));
        
        // Calculate margin level
        let margin_level = updated_account.calculate_margin_level(btc_price);
        
        // Equity: 0.1 BTC * $50,000 + $1,000 = $6,000
        // Borrowed: 0.1 BTC * $50,000 = $5,000
        // Margin level: $6,000 / $5,000 = 1.2 = 120%
        assert_eq!(margin_level, dec!(1.2));
        
        // Check for margin calls
        let calls = margin_manager.check_margin_calls(btc_price).await.unwrap();
        
        // Margin level is below maintenance (150%) but above liquidation (110%)
        // Should receive a maintenance margin call
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call_type, MarginCallType::MaintenanceCall);
        
        // Add more collateral to resolve the margin call
        let account_id = account.id;
        let account = margin_manager.get_account(&account_id).await.unwrap();
        
        // Update the quote balance (add more USDT)
        let mut updated_account = account.clone();
        updated_account.quote_balance += dec!(2000); // Add $2,000
        margin_manager.update_account(updated_account).await.unwrap();
        
        // Check margin level again
        margin_manager.update_margin_level(&account_id, btc_price).await.unwrap();
        
        // Check for margin calls again
        let calls = margin_manager.check_margin_calls(btc_price).await.unwrap();
        
        // Should be no new margin calls
        assert_eq!(calls.len(), 0);
        
        // Verify account status is back to Active
        let account = margin_manager.get_account(&account_id).await.unwrap();
        assert_eq!(account.status, MarginAccountStatus::Active);
        
        // Calculate new margin level
        // Equity: 0.1 BTC * $50,000 + $3,000 = $8,000
        // Borrowed: 0.1 BTC * $50,000 = $5,000
        // Margin level: $8,000 / $5,000 = 1.6 = 160%
        let margin_level = account.calculate_margin_level(btc_price);
        assert_eq!(margin_level, dec!(1.6));
    }
    
    #[tokio::test]
    async fn test_liquidation() {
        // Create margin account manager
        let margin_manager = MarginAccountManager::new(
            dec!(1.5),  // 150% maintenance margin requirement
            dec!(1.1),  // 110% liquidation threshold
        );
        
        // Create account
        let user_id = Uuid::new_v4();
        let account = margin_manager.create_account(
            user_id,
            true, // isolated margin
            "BTC".to_string(),
            "USDT".to_string(),
            dec!(0),    // 0 BTC initial
            dec!(5500), // $5,500 USDT initial
            dec!(0.05), // 5% annual interest on BTC
            dec!(0.03), // 3% annual interest on USDT
        ).await.unwrap();
        
        // Borrow BTC
        let initial_btc_price = dec!(50000); // $50,000 per BTC
        margin_manager.borrow(&account.id, true, dec!(0.1)).await.unwrap(); // Borrow 0.1 BTC
        
        // Update margin level
        margin_manager.update_margin_level(&account.id, initial_btc_price).await.unwrap();
        
        // Price drops significantly
        let crashed_price = dec!(45000); // $45,000 per BTC
        
        // Check for margin calls
        let calls = margin_manager.check_margin_calls(crashed_price).await.unwrap();
        
        // Margin level calculation:
        // Equity: 0.1 BTC * $45,000 + $5,500 = $10,000
        // Borrowed: 0.1 BTC * $45,000 = $4,500
        // Margin level: $10,000 / $4,500 = 2.22 = 222%
        // Still above maintenance, no margin call expected
        assert_eq!(calls.len(), 0);
        
        // Price crashes further and user withdraws funds
        let mut account = margin_manager.get_account(&account.id).await.unwrap();
        account.quote_balance = dec!(1000); // Reduce USDT to $1,000
        margin_manager.update_account(account).await.unwrap();
        
        let further_crashed_price = dec!(35000); // $35,000 per BTC
        
        // Check for margin calls
        let calls = margin_manager.check_margin_calls(further_crashed_price).await.unwrap();
        
        // Margin level calculation:
        // Equity: 0.1 BTC * $35,000 + $1,000 = $4,500
        // Borrowed: 0.1 BTC * $35,000 = $3,500
        // Margin level: $4,500 / $3,500 = 1.29 = 129%
        // Below maintenance (150%) but above liquidation (110%)
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call_type, MarginCallType::MaintenanceCall);
        
        // Price crashes even more
        let liquidation_price = dec!(30000); // $30,000 per BTC
        
        // Check for margin calls
        let calls = margin_manager.check_margin_calls(liquidation_price).await.unwrap();
        
        // Margin level calculation:
        // Equity: 0.1 BTC * $30,000 + $1,000 = $4,000
        // Borrowed: 0.1 BTC * $30,000 = $3,000
        // Margin level: $4,000 / $3,000 = 1.33 = 133%
        // Still just a maintenance call, not liquidation yet
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call_type, MarginCallType::MaintenanceCall);
        
        // Price crashes to liquidation level
        let severe_crash_price = dec!(25000); // $25,000 per BTC
        
        // Check for margin calls
        let calls = margin_manager.check_margin_calls(severe_crash_price).await.unwrap();
        
        // Margin level calculation:
        // Equity: 0.1 BTC * $25,000 + $1,000 = $3,500
        // Borrowed: 0.1 BTC * $25,000 = $2,500
        // Margin level: $3,500 / $2,500 = 1.4 = 140%
        // Still above liquidation threshold
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call_type, MarginCallType::MaintenanceCall);
        
        // Price crashes severely and user withdraws more funds
        let mut account = margin_manager.get_account(&account.id).await.unwrap();
        account.quote_balance = dec!(100); // Reduce USDT to $100
        margin_manager.update_account(account).await.unwrap();
        
        let extreme_crash_price = dec!(20000); // $20,000 per BTC
        
        // Check for margin calls
        let calls = margin_manager.check_margin_calls(extreme_crash_price).await.unwrap();
        
        // Margin level calculation:
        // Equity: 0.1 BTC * $20,000 + $100 = $2,100
        // Borrowed: 0.1 BTC * $20,000 = $2,000
        // Margin level: $2,100 / $2,000 = 1.05 = 105%
        // Below liquidation threshold (110%)
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call_type, MarginCallType::LiquidationCall);
        
        // Get account status
        let account = margin_manager.get_account(&account.id).await.unwrap();
        assert_eq!(account.status, MarginAccountStatus::Liquidating);
        
        // Liquidate the account
        margin_manager.liquidate_account(&account.id).await.unwrap();
        
        // Check account status after liquidation
        let account = margin_manager.get_account(&account.id).await.unwrap();
        assert_eq!(account.status, MarginAccountStatus::Closed);
        assert_eq!(account.base_balance, dec!(0));
        assert_eq!(account.quote_balance, dec!(0));
        assert_eq!(account.borrowed_base, dec!(0));
        assert_eq!(account.borrowed_quote, dec!(0));
    }
}
