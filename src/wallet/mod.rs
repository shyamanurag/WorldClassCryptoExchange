

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use hex;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

// Define core types
pub type UserId = Uuid;
pub type AssetId = String;
pub type WalletId = Uuid;
pub type TransactionId = Uuid;
pub type Address = String;
pub type PrivateKey = String;
pub type PublicKey = String;
pub type BlockchainId = String;
pub type NetworkFee = Decimal;

/// Error types for wallet operations
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Insufficient balance: requested {requested}, available {available}")]
    InsufficientBalance {
        requested: Decimal,
        available: Decimal,
    },
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Transaction processing error: {0}")]
    TransactionProcessingError(String),
    
    #[error("Cold storage error: {0}")]
    ColdStorageError(String),
    
    #[error("Hot wallet error: {0}")]
    HotWalletError(String),
    
    #[error("Security error: {0}")]
    SecurityError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Operation not permitted: {0}")]
    OperationNotPermitted(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Type of wallet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletType {
    Hot,    // Connected to internet, for immediate operations
    Cold,   // Offline storage for security
    User,   // User wallet for trading
    System, // System wallet for fees, etc.
}

/// Status of a wallet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletStatus {
    Active,
    Suspended,
    Locked,
    Closed,
}

/// Type of transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
    Fee,
    Trade,
    Staking,
    Unstaking,
    Reward,
    Refund,
    Adjustment,
}

/// Status of a transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
    Rejected,
    Processing,
}

/// A blockchain asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub symbol: String,
    pub name: String,
    pub blockchain_id: BlockchainId,
    pub decimals: u8,
    pub is_active: bool,
    pub min_deposit: Decimal,
    pub min_withdrawal: Decimal,
    pub withdrawal_fee: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A user wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: WalletId,
    pub user_id: Option<UserId>,
    pub wallet_type: WalletType,
    pub status: WalletStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Balance for a specific asset in a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub wallet_id: WalletId,
    pub asset_id: AssetId,
    pub total: Decimal,
    pub available: Decimal,
    pub reserved: Decimal,
    pub updated_at: DateTime<Utc>,
}

impl Balance {
    pub fn new(wallet_id: WalletId, asset_id: AssetId) -> Self {
        Balance {
            wallet_id,
            asset_id,
            total: Decimal::ZERO,
            available: Decimal::ZERO,
            reserved: Decimal::ZERO,
            updated_at: Utc::now(),
        }
    }
    
    pub fn credit(&mut self, amount: Decimal) {
        self.total += amount;
        self.available += amount;
        self.updated_at = Utc::now();
    }
    
    pub fn debit(&mut self, amount: Decimal) -> Result<(), WalletError> {
        if self.available < amount {
            return Err(WalletError::InsufficientBalance {
                requested: amount,
                available: self.available,
            });
        }
        
        self.total -= amount;
        self.available -= amount;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn reserve(&mut self, amount: Decimal) -> Result<(), WalletError> {
        if self.available < amount {
            return Err(WalletError::InsufficientBalance {
                requested: amount,
                available: self.available,
            });
        }
        
        self.available -= amount;
        self.reserved += amount;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn release_reservation(&mut self, amount: Decimal) -> Result<(), WalletError> {
        if self.reserved < amount {
            return Err(WalletError::InsufficientBalance {
                requested: amount,
                available: self.reserved,
            });
        }
        
        self.available += amount;
        self.reserved -= amount;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn settle_reservation(&mut self, amount: Decimal) -> Result<(), WalletError> {
        if self.reserved < amount {
            return Err(WalletError::InsufficientBalance {
                requested: amount,
                available: self.reserved,
            });
        }
        
        self.total -= amount;
        self.reserved -= amount;
        self.updated_at = Utc::now();
        
        Ok(())
    }
}

/// A transaction in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionId,
    pub transaction_type: TransactionType,
    pub source_wallet_id: Option<WalletId>,
    pub destination_wallet_id: Option<WalletId>,
    pub asset_id: AssetId,
    pub amount: Decimal,
    pub fee: Decimal,
    pub status: TransactionStatus,
    pub blockchain_txid: Option<String>,
    pub address: Option<String>,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        source_wallet_id: Option<WalletId>,
        destination_wallet_id: Option<WalletId>,
        asset_id: AssetId,
        amount: Decimal,
        fee: Decimal,
        address: Option<String>,
        memo: Option<String>,
    ) -> Self {
        let now = Utc::now();
        
        Transaction {
            id: Uuid::new_v4(),
            transaction_type,
            source_wallet_id,
            destination_wallet_id,
            asset_id,
            amount,
            fee,
            status: TransactionStatus::Pending,
            blockchain_txid: None,
            address,
            memo,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }
    
    pub fn complete(&mut self, blockchain_txid: Option<String>) {
        self.status = TransactionStatus::Completed;
        self.blockchain_txid = blockchain_txid;
        self.updated_at = Utc::now();
        self.completed_at = Some(Utc::now());
    }
    
    pub fn fail(&mut self, reason: Option<String>) {
        self.status = TransactionStatus::Failed;
        self.memo = reason.or(self.memo.clone());
        self.updated_at = Utc::now();
    }
    
    pub fn cancel(&mut self) {
        self.status = TransactionStatus::Cancelled;
        self.updated_at = Utc::now();
    }
    
    pub fn reject(&mut self, reason: Option<String>) {
        self.status = TransactionStatus::Rejected;
        self.memo = reason.or(self.memo.clone());
        self.updated_at = Utc::now();
    }
    
    pub fn is_finalized(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Completed
                | TransactionStatus::Failed
                | TransactionStatus::Cancelled
                | TransactionStatus::Rejected
        )
    }
}

/// Address information for a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub wallet_id: WalletId,
    pub asset_id: AssetId,
    pub address: Address,
    pub memo: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Interface for blockchain adapters
#[async_trait]
pub trait BlockchainAdapter: Send + Sync {
    async fn generate_address(&self, asset_id: &AssetId) -> Result<Address, WalletError>;
    
    async fn validate_address(&self, asset_id: &AssetId, address: &Address) -> Result<bool, WalletError>;
    
    async fn get_network_fee(&self, asset_id: &AssetId) -> Result<NetworkFee, WalletError>;
    
    async fn broadcast_transaction(
        &self,
        asset_id: &AssetId,
        to_address: &Address,
        amount: Decimal,
        fee: NetworkFee,
        memo: Option<String>,
    ) -> Result<String, WalletError>;
    
    async fn get_transaction_status(&self, asset_id: &AssetId, txid: &str) -> Result<TransactionStatus, WalletError>;
    
    async fn get_balance(&self, asset_id: &AssetId, address: &Address) -> Result<Decimal, WalletError>;
}

/// In-memory implementation of an asset store
pub struct AssetStore {
    assets: RwLock<HashMap<AssetId, Asset>>,
}

impl AssetStore {
    pub fn new() -> Self {
        AssetStore {
            assets: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn add_asset(&self, asset: Asset) -> Result<(), WalletError> {
        let mut assets = self.assets.write().await;
        if assets.contains_key(&asset.id) {
            return Err(WalletError::InvalidParameters(format!("Asset already exists: {}", asset.id)));
        }
        
        assets.insert(asset.id.clone(), asset);
        Ok(())
    }
    
    pub async fn get_asset(&self, asset_id: &AssetId) -> Result<Asset, WalletError> {
        let assets = self.assets.read().await;
        assets
            .get(asset_id)
            .cloned()
            .ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))
    }
    
    pub async fn update_asset(&self, asset: Asset) -> Result<(), WalletError> {
        let mut assets = self.assets.write().await;
        if !assets.contains_key(&asset.id) {
            return Err(WalletError::AssetNotFound(asset.id.clone()));
        }
        
        assets.insert(asset.id.clone(), asset);
        Ok(())
    }
    
    pub async fn list_assets(&self) -> Vec<Asset> {
        let assets = self.assets.read().await;
        assets.values().cloned().collect()
    }
}

/// In-memory implementation of a wallet store
pub struct WalletStore {
    wallets: RwLock<HashMap<WalletId, Wallet>>,
    user_wallets: RwLock<HashMap<UserId, Vec<WalletId>>>,
    balances: RwLock<HashMap<(WalletId, AssetId), Balance>>,
    addresses: RwLock<HashMap<(WalletId, AssetId), Vec<AddressInfo>>>,
}

impl WalletStore {
    pub fn new() -> Self {
        WalletStore {
            wallets: RwLock::new(HashMap::new()),
            user_wallets: RwLock::new(HashMap::new()),
            balances: RwLock::new(HashMap::new()),
            addresses: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn create_wallet(&self, wallet_type: WalletType, user_id: Option<UserId>) -> Result<Wallet, WalletError> {
        let wallet_id = Uuid::new_v4();
        let now = Utc::now();
        
        let wallet = Wallet {
            id: wallet_id,
            user_id,
            wallet_type,
            status: WalletStatus::Active,
            created_at: now,
            updated_at: now,
        };
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(wallet_id, wallet.clone());
        
        if let Some(user_id) = user_id {
            let mut user_wallets = self.user_wallets.write().await;
            user_wallets
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(wallet_id);
        }
        
        Ok(wallet)
    }
    
    pub async fn get_wallet(&self, wallet_id: &WalletId) -> Result<Wallet, WalletError> {
        let wallets = self.wallets.read().await;
        wallets
            .get(wallet_id)
            .cloned()
            .ok_or_else(|| WalletError::WalletNotFound(wallet_id.to_string()))
    }
    
    pub async fn get_user_wallets(&self, user_id: &UserId) -> Vec<Wallet> {
        let user_wallets = self.user_wallets.read().await;
        let wallet_ids = match user_wallets.get(user_id) {
            Some(ids) => ids,
            None => return Vec::new(),
        };
        
        let wallets = self.wallets.read().await;
        wallet_ids
            .iter()
            .filter_map(|id| wallets.get(id).cloned())
            .collect()
    }
    
    pub async fn update_wallet_status(&self, wallet_id: &WalletId, status: WalletStatus) -> Result<Wallet, WalletError> {
        let mut wallets = self.wallets.write().await;
        let wallet = wallets
            .get_mut(wallet_id)
            .ok_or_else(|| WalletError::WalletNotFound(wallet_id.to_string()))?;
        
        wallet.status = status;
        wallet.updated_at = Utc::now();
        
        Ok(wallet.clone())
    }
    
    pub async fn get_balance(&self, wallet_id: &WalletId, asset_id: &AssetId) -> Result<Balance, WalletError> {
        // Check if wallet exists
        let _ = self.get_wallet(wallet_id).await?;
        
        let balances = self.balances.read().await;
        let key = (wallet_id.clone(), asset_id.clone());
        
        match balances.get(&key) {
            Some(balance) => Ok(balance.clone()),
            None => Ok(Balance::new(wallet_id.clone(), asset_id.clone())),
        }
    }
    
    pub async fn update_balance(&self, balance: Balance) -> Result<(), WalletError> {
        // Check if wallet exists
        let _ = self.get_wallet(&balance.wallet_id).await?;
        
        let mut balances = self.balances.write().await;
        let key = (balance.wallet_id, balance.asset_id.clone());
        
        balances.insert(key, balance);
        Ok(())
    }
    
    pub async fn add_address(&self, address_info: AddressInfo) -> Result<(), WalletError> {
        // Check if wallet exists
        let _ = self.get_wallet(&address_info.wallet_id).await?;
        
        let mut addresses = self.addresses.write().await;
        let key = (address_info.wallet_id, address_info.asset_id.clone());
        
        addresses
            .entry(key)
            .or_insert_with(Vec::new)
            .push(address_info);
        
        Ok(())
    }
    
    pub async fn get_active_address(&self, wallet_id: &WalletId, asset_id: &AssetId) -> Result<AddressInfo, WalletError> {
        // Check if wallet exists
        let _ = self.get_wallet(wallet_id).await?;
        
        let addresses = self.addresses.read().await;
        let key = (wallet_id.clone(), asset_id.clone());
        
        if let Some(addr_list) = addresses.get(&key) {
            for addr in addr_list.iter().rev() {
                if addr.is_active {
                    return Ok(addr.clone());
                }
            }
        }
        
        Err(WalletError::InvalidAddress(format!(
            "No active address found for wallet {} and asset {}",
            wallet_id, asset_id
        )))
    }
    
    pub async fn list_addresses(&self, wallet_id: &WalletId, asset_id: &AssetId) -> Result<Vec<AddressInfo>, WalletError> {
        // Check if wallet exists
        let _ = self.get_wallet(wallet_id).await?;
        
        let addresses = self.addresses.read().await;
        let key = (wallet_id.clone(), asset_id.clone());
        
        match addresses.get(&key) {
            Some(addr_list) => Ok(addr_list.clone()),
            None => Ok(Vec::new()),
        }
    }
}

/// In-memory implementation of a transaction store
pub struct TransactionStore {
    transactions: RwLock<HashMap<TransactionId, Transaction>>,
    wallet_transactions: RwLock<HashMap<WalletId, Vec<TransactionId>>>,
}

impl TransactionStore {
    pub fn new() -> Self {
        TransactionStore {
            transactions: RwLock::new(HashMap::new()),
            wallet_transactions: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn add_transaction(&self, transaction: Transaction) -> Result<(), WalletError> {
        let mut transactions = self.transactions.write().await;
        
        if let Some(source_wallet_id) = transaction.source_wallet_id {
            let mut wallet_transactions = self.wallet_transactions.write().await;
            wallet_transactions
                .entry(source_wallet_id)
                .or_insert_with(Vec::new)
                .push(transaction.id);
        }
        
        if let Some(dest_wallet_id) = transaction.destination_wallet_id {
            let mut wallet_transactions = self.wallet_transactions.write().await;
            wallet_transactions
                .entry(dest_wallet_id)
                .or_insert_with(Vec::new)
                .push(transaction.id);
        }
        
        transactions.insert(transaction.id, transaction);
        Ok(())
    }
    
    pub async fn get_transaction(&self, transaction_id: &TransactionId) -> Result<Transaction, WalletError> {
        let transactions = self.transactions.read().await;
        transactions
            .get(transaction_id)
            .cloned()
            .ok_or_else(|| WalletError::TransactionNotFound(transaction_id.to_string()))
    }
    
    pub async fn update_transaction(&self, transaction: Transaction) -> Result<(), WalletError> {
        let mut transactions = self.transactions.write().await;
        if !transactions.contains_key(&transaction.id) {
            return Err(WalletError::TransactionNotFound(transaction.id.to_string()));
        }
        
        transactions.insert(transaction.id, transaction);
        Ok(())
    }
    
    pub async fn get_wallet_transactions(&self, wallet_id: &WalletId) -> Vec<Transaction> {
        let wallet_transactions = self.wallet_transactions.read().await;
        let transaction_ids = match wallet_transactions.get(wallet_id) {
            Some(ids) => ids,
            None => return Vec::new(),
        };
        
        let transactions = self.transactions.read().await;
        transaction_ids
            .iter()
            .filter_map(|id| transactions.get(id).cloned())
            .collect()
    }
    
    pub async fn get_pending_transactions(&self, asset_id: &AssetId) -> Vec<Transaction> {
        let transactions = self.transactions.read().await;
        transactions
            .values()
            .filter(|tx| tx.asset_id == *asset_id && tx.status == TransactionStatus::Pending)
            .cloned()
            .collect()
    }
}

/// A simple implementation of a blockchain adapter (mock for testing)
pub struct MockBlockchainAdapter {
    asset_id: AssetId,
    network_fee: NetworkFee,
}

impl MockBlockchainAdapter {
    pub fn new(asset_id: AssetId, network_fee: NetworkFee) -> Self {
        MockBlockchainAdapter {
            asset_id,
            network_fee,
        }
    }
    
    fn generate_mock_address(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("addr:{}-{}", self.asset_id, Uuid::new_v4()));
        let result = hasher.finalize();
        format!("{}-{}", self.asset_id, hex::encode(&result[0..20]))
    }
}

#[async_trait]
impl BlockchainAdapter for MockBlockchainAdapter {
    async fn generate_address(&self, asset_id: &AssetId) -> Result<Address, WalletError> {
        if asset_id != &self.asset_id {
            return Err(WalletError::AssetNotFound(asset_id.clone()));
        }
        
        Ok(self.generate_mock_address())
    }
    
    async fn validate_address(&self, asset_id: &AssetId, address: &Address) -> Result<bool, WalletError> {
        if asset_id != &self.asset_id {
            return Err(WalletError::AssetNotFound(asset_id.clone()));
        }
        
        // Simple validation: check if address starts with asset_id
        Ok(address.starts_with(&format!("{}-", self.asset_id)))
    }
    
    async fn get_network_fee(&self, asset_id: &AssetId) -> Result<NetworkFee, WalletError> {
        if asset_id != &self.asset_id {
            return Err(WalletError::AssetNotFound(asset_id.clone()));
        }
        
        Ok(self.network_fee)
    }
    
    async fn broadcast_transaction(
        &self,
        asset_id: &AssetId,
        to_address: &Address,
        amount: Decimal,
        fee: NetworkFee,
        memo: Option<String>,
    ) -> Result<String, WalletError> {
        if asset_id != &self.asset_id {
            return Err(WalletError::AssetNotFound(asset_id.clone()));
        }
        
        // Validate address
        if !self.validate_address(asset_id, to_address).await? {
            return Err(WalletError::InvalidAddress(to_address.clone()));
        }
        
        // Generate mock transaction hash
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "tx:{}:{}:{}:{}:{}",
            asset_id,
            to_address,
            amount,
            fee,
            memo.unwrap_or_default()
        ));
        let result = hasher.finalize();
        
        Ok(hex::encode(result))
    }
    
    async fn get_transaction_status(&self, asset_id: &AssetId, txid: &str) -> Result<TransactionStatus, WalletError> {
        if asset_id != &self.asset_id {
            return Err(WalletError::AssetNotFound(asset_id.clone()));
        }
        
        // For mock purposes, consider all transactions confirmed after validation
        Ok(TransactionStatus::Completed)
    }
    
    async fn get_balance(&self, asset_id: &AssetId, address: &Address) -> Result<Decimal, WalletError> {
        if asset_id != &self.asset_id {
            return Err(WalletError::AssetNotFound(asset_id.clone()));
        }
        
        // Mock balance for testing
        Ok(Decimal::from(1000))
    }
}

/// Hot wallet implementation for handling daily operations
pub struct HotWallet {
    asset_id: AssetId,
    wallet_id: WalletId,
    wallet_store: Arc<WalletStore>,
    transaction_store: Arc<TransactionStore>,
    blockchain_adapter: Arc<dyn BlockchainAdapter>,
    withdrawal_limit: Decimal,
}

impl HotWallet {
    pub fn new(
        asset_id: AssetId,
        wallet_id: WalletId,
        wallet_store: Arc<WalletStore>,
        transaction_store: Arc<TransactionStore>,
        blockchain_adapter: Arc<dyn BlockchainAdapter>,
        withdrawal_limit: Decimal,
    ) -> Self {
        HotWallet {
            asset_id,
            wallet_id,
            wallet_store,
            transaction_store,
            blockchain_adapter,
            withdrawal_limit,
        }
    }
    
    pub async fn generate_deposit_address(&self, user_wallet_id: &WalletId) -> Result<AddressInfo, WalletError> {
        // Verify user wallet exists
        let user_wallet = self.wallet_store.get_wallet(user_wallet_id).await?;
        
        // Generate address
        let address = self.blockchain_adapter.generate_address(&self.asset_id).await?;
        
        // Create address info
        let address_info = AddressInfo {
            wallet_id: user_wallet.id,
            asset_id: self.asset_id.clone(),
            address,
            memo: None,
            is_active: true,
            created_at: Utc::now(),
        };
        
        // Save address
        self.wallet_store.add_address(address_info.clone()).await?;
        
        Ok(address_info)
    }
    
    pub async fn process_deposit(
        &self,
        user_wallet_id: &WalletId,
        amount: Decimal,
        blockchain_txid: String,
    ) -> Result<Transaction, WalletError> {
        // Verify user wallet exists and is active
        let user_wallet = self.wallet_store.get_wallet(user_wallet_id).await?;
        if user_wallet.status != WalletStatus::Active {
            return Err(WalletError::OperationNotPermitted(format!(
                "Wallet is not active: {:?}",
                user_wallet.status
            )));
        }
        
        // Create transaction
        let transaction = Transaction::new(
            TransactionType::Deposit,
            Some(self.wallet_id),
            Some(user_wallet.id),
            self.asset_id.clone(),
            amount,
            Decimal::ZERO,
            None,
            None,
        );
        
        // Update transaction with blockchain info
        let mut updated_tx = transaction.clone();
        updated_tx.blockchain_txid = Some(blockchain_txid);
        updated_tx.status = TransactionStatus::Processing;
        
        // Save transaction
        self.transaction_store.add_transaction(updated_tx.clone()).await?;
        
        // Update balances
        // First debit from hot wallet
        let mut hot_balance = self.wallet_store.get_balance(&self.wallet_id, &self.asset_id).await?;
        hot_balance.debit(amount)?;
        self.wallet_store.update_balance(hot_balance).await?;
        
        // Then credit to user wallet
        let mut user_balance = self.wallet_store.get_balance(user_wallet_id, &self.asset_id).await?;
        user_balance.credit(amount);
        self.wallet_store.update_balance(user_balance).await?;
        
        // Complete transaction
        updated_tx.complete(Some(blockchain_txid));
        self.transaction_store.update_transaction(updated_tx.clone()).await?;
        
        Ok(updated_tx)
    }
    
    pub async fn process_withdrawal(
        &self,
        user_wallet_id: &WalletId,
        address: &str,
        amount: Decimal,
        memo: Option<String>,
    ) -> Result<Transaction, WalletError> {
        // Verify user wallet exists and is active
        let user_wallet = self.wallet_store.get_wallet(user_wallet_id).await?;
        if user_wallet.status != WalletStatus::Active {
            return Err(WalletError::OperationNotPermitted(format!(
                "Wallet is not active: {:?}",
                user_wallet.status
            )));
        }
        
        // Validate address
        if !self.blockchain_adapter.validate_address(&self.asset_id, address).await? {
            return Err(WalletError::InvalidAddress(address.to_string()));
        }
        
        // Check withdrawal limit
        if amount > self.withdrawal_limit {
            return Err(WalletError::OperationNotPermitted(format!(
                "Withdrawal amount exceeds limit: {} > {}",
                amount, self.withdrawal_limit
            )));
        }
        
        // Get network fee
        let network_fee = self.blockchain_adapter.get_network_fee(&self.asset_id).await?;
        
        // Check if user has enough balance
        let mut user_balance = self.wallet_store.get_balance(user_wallet_id, &self.asset_id).await?;
        let total_amount = amount + network_fee;
        
        if user_balance.available < total_amount {
            return Err(WalletError::InsufficientBalance {
                requested: total_amount,
                available: user_balance.available,
            });
        }
        
        // Reserve the amount in user's wallet
        user_balance.reserve(total_amount)?;
        self.wallet_store.update_balance(user_balance.clone()).await?;
        
        // Create transaction
        let transaction = Transaction::new(
            TransactionType::Withdrawal,
            Some(user_wallet.id),
            Some(self.wallet_id),
            self.asset_id.clone(),
            amount,
            network_fee,
            Some(address.to_string()),
            memo.clone(),
        );
        
        // Save transaction
        self.transaction_store.add_transaction(transaction.clone()).await?;
        
        // Broadcast to blockchain
        let blockchain_txid = match self
            .blockchain_adapter
            .broadcast_transaction(&self.asset_id, address, amount, network_fee, memo)
            .await
        {
            Ok(txid) => txid,
            Err(err) => {
                // If broadcasting fails, release the reservation
                user_balance.release_reservation(total_amount)?;
                self.wallet_store.update_balance(user_balance).await?;
                
                // Update transaction to failed
                let mut failed_tx = transaction.clone();
                failed_tx.fail(Some(format!("Broadcast failed: {}", err)));
                self.transaction_store.update_transaction(failed_tx).await?;
                
                return Err(err);
            }
        };
        
        // Settle the reservation (deduct from user's wallet)
        user_balance.settle_reservation(total_amount)?;
        self.wallet_store.update_balance(user_balance).await?;
        
        // Credit the hot wallet (minus network fee)
        let mut hot_balance = self.wallet_store.get_balance(&self.wallet_id, &self.asset_id).await?;
        hot_balance.credit(amount);
        self.wallet_store.update_balance(hot_balance).await?;
        
        // Update and complete the transaction
        let mut completed_tx = transaction.clone();
        completed_tx.complete(Some(blockchain_txid.clone()));
        self.transaction_store.update_transaction(completed_tx.clone()).await?;
        
        Ok(completed_tx)
    }
    
    pub async fn get_hot_wallet_balance(&self) -> Result<Balance, WalletError> {
        self.wallet_store.get_balance(&self.wallet_id, &self.asset_id).await
    }
}

/// Cold storage for secure asset storage
pub struct ColdStorage {
    asset_id: AssetId,
    wallet_id: WalletId,
    wallet_store: Arc<WalletStore>,
    transaction_store: Arc<TransactionStore>,
    blockchain_adapter: Arc<dyn BlockchainAdapter>,
    threshold_amount: Decimal,
    approval_required: bool,
}

impl ColdStorage {
    pub fn new(
        asset_id: AssetId,
        wallet_id: WalletId,
        wallet_store: Arc<WalletStore>,
        transaction_store: Arc<TransactionStore>,
        blockchain_adapter: Arc<dyn BlockchainAdapter>,
        threshold_amount: Decimal,
        approval_required: bool,
    ) -> Self {
        ColdStorage {
            asset_id,
            wallet_id,
            wallet_store,
            transaction_store,
            blockchain_adapter,
            threshold_amount,
            approval_required,
        }
    }
    
    pub async fn get_deposit_address(&self) -> Result<AddressInfo, WalletError> {
        // Try to get an existing active address first
        match self.wallet_store.get_active_address(&self.wallet_id, &self.asset_id).await {
            Ok(address) => Ok(address),
            Err(_) => {
                // Generate a new address
                let address = self.blockchain_adapter.generate_address(&self.asset_id).await?;
                
                // Create address info
                let address_info = AddressInfo {
                    wallet_id: self.wallet_id,
                    asset_id: self.asset_id.clone(),
                    address,
                    memo: None,
                    is_active: true,
                    created_at: Utc::now(),
                };
                
                // Save address
                self.wallet_store.add_address(address_info.clone()).await?;
                
                Ok(address_info)
            }
        }
    }
    
    pub async fn transfer_to_cold_storage(
        &self,
        hot_wallet_id: &WalletId,
        amount: Decimal,
    ) -> Result<Transaction, WalletError> {
        // Verify hot wallet exists
        let hot_wallet = self.wallet_store.get_wallet(hot_wallet_id).await?;
        
        // Check if amount exceeds threshold
        if amount < self.threshold_amount {
            return Err(WalletError::InvalidParameters(format!(
                "Amount below threshold: {} < {}",
                amount, self.threshold_amount
            )));
        }
        
        // Check hot wallet balance
        let mut hot_balance = self.wallet_store.get_balance(hot_wallet_id, &self.asset_id).await?;
        if hot_balance.available < amount {
            return Err(WalletError::InsufficientBalance {
                requested: amount,
                available: hot_balance.available,
            });
        }
        
        // Get cold storage address
        let address_info = self.get_deposit_address().await?;
        
        // Create transaction
        let mut transaction = Transaction::new(
            TransactionType::Transfer,
            Some(hot_wallet.id),
            Some(self.wallet_id),
            self.asset_id.clone(),
            amount,
            Decimal::ZERO,
            Some(address_info.address.clone()),
            None,
        );
        
        // If approval is required, set status to pending
        if self.approval_required {
            transaction.status = TransactionStatus::Pending;
            self.transaction_store.add_transaction(transaction.clone()).await?;
            return Ok(transaction);
        }
        
        // Reserve amount in hot wallet
        hot_balance.reserve(amount)?;
        self.wallet_store.update_balance(hot_balance.clone()).await?;
        
        // Save transaction as processing
        transaction.status = TransactionStatus::Processing;
        self.transaction_store.add_transaction(transaction.clone()).await?;
        
        // Get network fee
        let network_fee = self.blockchain_adapter.get_network_fee(&self.asset_id).await?;
        
        // Broadcast to blockchain
        let blockchain_txid = match self
            .blockchain_adapter
            .broadcast_transaction(
                &self.asset_id,
                &address_info.address,
                amount,
                network_fee,
                None,
            )
            .await
        {
            Ok(txid) => txid,
            Err(err) => {
                // If broadcasting fails, release the reservation
                hot_balance.release_reservation(amount)?;
                self.wallet_store.update_balance(hot_balance).await?;
                
                // Update transaction to failed
                let mut failed_tx = transaction.clone();
                failed_tx.fail(Some(format!("Broadcast failed: {}", err)));
                self.transaction_store.update_transaction(failed_tx).await?;
                
                return Err(err);
            }
        };
        
        // Settle the reservation in hot wallet
        hot_balance.settle_reservation(amount)?;
        self.wallet_store.update_balance(hot_balance).await?;
        
        // Credit cold storage wallet
        let mut cold_balance = self.wallet_store.get_balance(&self.wallet_id, &self.asset_id).await?;
        cold_balance.credit(amount - network_fee); // Subtract network fee
        self.wallet_store.update_balance(cold_balance).await?;
        
        // Update and complete the transaction
        let mut completed_tx = transaction.clone();
        completed_tx.fee = network_fee; // Update with actual fee
        completed_tx.complete(Some(blockchain_txid.clone()));
        self.transaction_store.update_transaction(completed_tx.clone()).await?;
        
        Ok(completed_tx)
    }
    
    pub async fn transfer_from_cold_storage(
        &self,
        hot_wallet_id: &WalletId,
        amount: Decimal,
        approval_signatures: Vec<String>,
    ) -> Result<Transaction, WalletError> {
        // This would involve a complex multi-signature process in a real implementation
        // For this example, we'll simplify and just check that there are enough approvals
        
        if self.approval_required && approval_signatures.len() < 3 {
            return Err(WalletError::SecurityError(
                "Insufficient approvals for cold storage transfer".to_string(),
            ));
        }
        
        // Verify hot wallet exists
        let hot_wallet = self.wallet_store.get_wallet(hot_wallet_id).await?;
        
        // Check cold storage balance
        let mut cold_balance = self.wallet_store.get_balance(&self.wallet_id, &self.asset_id).await?;
        if cold_balance.available < amount {
            return Err(WalletError::InsufficientBalance {
                requested: amount,
                available: cold_balance.available,
            });
        }
        
        // Get hot wallet address
        let hot_wallet_address = match self.wallet_store.get_active_address(hot_wallet_id, &self.asset_id).await {
            Ok(address_info) => address_info.address,
            Err(_) => {
                return Err(WalletError::InvalidAddress(
                    "No active address found for hot wallet".to_string(),
                ))
            }
        };
        
        // Create transaction
        let mut transaction = Transaction::new(
            TransactionType::Transfer,
            Some(self.wallet_id),
            Some(hot_wallet.id),
            self.asset_id.clone(),
            amount,
            Decimal::ZERO,
            Some(hot_wallet_address.clone()),
            None,
        );
        
        // Reserve amount in cold storage
        cold_balance.reserve(amount)?;
        self.wallet_store.update_balance(cold_balance.clone()).await?;
        
        // Save transaction as processing
        transaction.status = TransactionStatus::Processing;
        self.transaction_store.add_transaction(transaction.clone()).await?;
        
        // Get network fee
        let network_fee = self.blockchain_adapter.get_network_fee(&self.asset_id).await?;
        
        // Broadcast to blockchain (in a real impl, this would use multi-sig)
        let blockchain_txid = match self
            .blockchain_adapter
            .broadcast_transaction(
                &self.asset_id,
                &hot_wallet_address,
                amount,
                network_fee,
                None,
            )
            .await
        {
            Ok(txid) => txid,
            Err(err) => {
                // If broadcasting fails, release the reservation
                cold_balance.release_reservation(amount)?;
                self.wallet_store.update_balance(cold_balance).await?;
                
                // Update transaction to failed
                let mut failed_tx = transaction.clone();
                failed_tx.fail(Some(format!("Broadcast failed: {}", err)));
                self.transaction_store.update_transaction(failed_tx).await?;
                
                return Err(err);
            }
        };
        
        // Settle the reservation in cold storage
        cold_balance.settle_reservation(amount)?;
        self.wallet_store.update_balance(cold_balance).await?;
        
        // Credit hot wallet
        let mut hot_balance = self.wallet_store.get_balance(hot_wallet_id, &self.asset_id).await?;
        hot_balance.credit(amount - network_fee); // Subtract network fee
        self.wallet_store.update_balance(hot_balance).await?;
        
        // Update and complete the transaction
        let mut completed_tx = transaction.clone();
        completed_tx.fee = network_fee; // Update with actual fee
        completed_tx.complete(Some(blockchain_txid.clone()));
        self.transaction_store.update_transaction(completed_tx.clone()).await?;
        
        Ok(completed_tx)
    }
    
    pub async fn get_cold_storage_balance(&self) -> Result<Balance, WalletError> {
        self.wallet_store.get_balance(&self.wallet_id, &self.asset_id).await
    }
}

/// Wallet system that manages all wallets and assets
pub struct WalletSystem {
    asset_store: Arc<AssetStore>,
    wallet_store: Arc<WalletStore>,
    transaction_store: Arc<TransactionStore>,
    hot_wallets: RwLock<HashMap<AssetId, Arc<HotWallet>>>,
    cold_storages: RwLock<HashMap<AssetId, Arc<ColdStorage>>>,
    blockchain_adapters: RwLock<HashMap<AssetId, Arc<dyn BlockchainAdapter>>>,
}

impl WalletSystem {
    pub fn new() -> Self {
        WalletSystem {
            asset_store: Arc::new(AssetStore::new()),
            wallet_store: Arc::new(WalletStore::new()),
            transaction_store: Arc::new(TransactionStore::new()),
            hot_wallets: RwLock::new(HashMap::new()),
            cold_storages: RwLock::new(HashMap::new()),
            blockchain_adapters: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn register_asset(&self, asset: Asset, blockchain_adapter: Arc<dyn BlockchainAdapter>) -> Result<(), WalletError> {
        // Add asset to store
        self.asset_store.add_asset(asset.clone()).await?;
        
        // Add blockchain adapter
        let mut adapters = self.blockchain_adapters.write().await;
        adapters.insert(asset.id.clone(), blockchain_adapter);
        
        Ok(())
    }
    
    pub async fn initialize_system_wallets(&self, asset_id: &AssetId, withdrawal_limit: Decimal, cold_threshold: Decimal) -> Result<(), WalletError> {
        // Get asset and blockchain adapter
        let asset = self.asset_store.get_asset(asset_id).await?;
        let adapters = self.blockchain_adapters.read().await;
        let blockchain_adapter = adapters
            .get(asset_id)
            .cloned()
            .ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))?;
        
        // Create hot wallet
        let hot_wallet_id = {
            let wallet = self.wallet_store.create_wallet(WalletType::Hot, None).await?;
            wallet.id
        };
        
        // Create cold storage wallet
        let cold_wallet_id = {
            let wallet = self.wallet_store.create_wallet(WalletType::Cold, None).await?;
            wallet.id
        };
        
        // Initialize hot wallet
        let hot_wallet = Arc::new(HotWallet::new(
            asset_id.clone(),
            hot_wallet_id,
            self.wallet_store.clone(),
            self.transaction_store.clone(),
            blockchain_adapter.clone(),
            withdrawal_limit,
        ));
        
        // Initialize cold storage
        let cold_storage = Arc::new(ColdStorage::new(
            asset_id.clone(),
            cold_wallet_id,
            self.wallet_store.clone(),
            self.transaction_store.clone(),
            blockchain_adapter.clone(),
            cold_threshold,
            true, // Require approval for cold storage operations
        ));
        
        // Register hot wallet and cold storage
        let mut hot_wallets = self.hot_wallets.write().await;
        hot_wallets.insert(asset_id.clone(), hot_wallet);
        
        let mut cold_storages = self.cold_storages.write().await;
        cold_storages.insert(asset_id.clone(), cold_storage);
        
        Ok(())
    }
    
    pub async fn create_user_wallet(&self, user_id: UserId) -> Result<Wallet, WalletError> {
        self.wallet_store.create_wallet(WalletType::User, Some(user_id)).await
    }
    
    pub async fn generate_deposit_address(&self, user_id: &UserId, asset_id: &AssetId) -> Result<AddressInfo, WalletError> {
        // Get user wallets
        let user_wallets = self.wallet_store.get_user_wallets(user_id).await;
        if user_wallets.is_empty() {
            return Err(WalletError::UserNotFound(user_id.to_string()));
        }
        
        // Find an active wallet
        let user_wallet = user_wallets.iter().find(|w| w.status == WalletStatus::Active).ok_or_else(|| {
            WalletError::OperationNotPermitted("No active wallet found for user".to_string())
        })?;
        
        // Get hot wallet for asset
        let hot_wallets = self.hot_wallets.read().await;
        let hot_wallet = hot_wallets.get(asset_id).ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))?;
        
        // Generate deposit address
        hot_wallet.generate_deposit_address(&user_wallet.id).await
    }
    
    pub async fn process_withdrawal(
        &self,
        user_id: &UserId,
        asset_id: &AssetId,
        address: &str,
        amount: Decimal,
        memo: Option<String>,
    ) -> Result<Transaction, WalletError> {
        // Get user wallets
        let user_wallets = self.wallet_store.get_user_wallets(user_id).await;
        if user_wallets.is_empty() {
            return Err(WalletError::UserNotFound(user_id.to_string()));
        }
        
        // Find an active wallet
        let user_wallet = user_wallets.iter().find(|w| w.status == WalletStatus::Active).ok_or_else(|| {
            WalletError::OperationNotPermitted("No active wallet found for user".to_string())
        })?;
        
        // Get hot wallet for asset
        let hot_wallets = self.hot_wallets.read().await;
        let hot_wallet = hot_wallets.get(asset_id).ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))?;
        
        // Process withdrawal
        hot_wallet.process_withdrawal(&user_wallet.id, address, amount, memo).await
    }
    
    pub async fn transfer_to_cold_storage(&self, asset_id: &AssetId, amount: Decimal) -> Result<Transaction, WalletError> {
        // Get hot wallet and cold storage for asset
        let hot_wallets = self.hot_wallets.read().await;
        let hot_wallet = hot_wallets.get(asset_id).ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))?;
        
        let cold_storages = self.cold_storages.read().await;
        let cold_storage = cold_storages.get(asset_id).ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))?;
        
        // Transfer to cold storage
        cold_storage.transfer_to_cold_storage(&hot_wallet.wallet_id, amount).await
    }
    
    pub async fn transfer_from_cold_storage(
        &self,
        asset_id: &AssetId,
        amount: Decimal,
        approval_signatures: Vec<String>,
    ) -> Result<Transaction, WalletError> {
        // Get hot wallet and cold storage for asset
        let hot_wallets = self.hot_wallets.read().await;
        let hot_wallet = hot_wallets.get(asset_id).ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))?;
        
        let cold_storages = self.cold_storages.read().await;
        let cold_storage = cold_storages.get(asset_id).ok_or_else(|| WalletError::AssetNotFound(asset_id.clone()))?;
        
        // Transfer from cold storage
        cold_storage
            .transfer_from_cold_storage(&hot_wallet.wallet_id, amount, approval_signatures)
            .await
    }
    
    pub async fn get_user_balance(&self, user_id: &UserId, asset_id: &AssetId) -> Result<Balance, WalletError> {
        // Get user wallets
        let user_wallets = self.wallet_store.get_user_wallets(user_id).await;
        if user_wallets.is_empty() {
            return Err(WalletError::UserNotFound(user_id.to_string()));
        }
        
        // Find an active wallet
        let user_wallet = user_wallets.iter().find(|w| w.status == WalletStatus::Active).ok_or_else(|| {
            WalletError::OperationNotPermitted("No active wallet found for user".to_string())
        })?;
        
        // Get balance
        self.wallet_store.get_balance(&user_wallet.id, asset_id).await
    }
    
    pub async fn get_user_transactions(&self, user_id: &UserId) -> Result<Vec<Transaction>, WalletError> {
        // Get user wallets
        let user_wallets = self.wallet_store.get_user_wallets(user_id).await;
        if user_wallets.is_empty() {
            return Err(WalletError::UserNotFound(user_id.to_string()));
        }
        
        // Get transactions for all user wallets
        let mut transactions = Vec::new();
        for wallet in user_wallets {
            let wallet_transactions = self.transaction_store.get_wallet_transactions(&wallet.id).await;
            transactions.extend(wallet_transactions);
        }
        
        // Sort by timestamp (newest first)
        transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(transactions)
    }
    
    pub async fn get_asset_balances(&self, user_id: &UserId) -> Result<Vec<(Asset, Balance)>, WalletError> {
        // Get all assets
        let assets = self.asset_store.list_assets().await;
        
        // Get balances for each asset
        let mut asset_balances = Vec::new();
        for asset in assets {
            match self.get_user_balance(user_id, &asset.id).await {
                Ok(balance) => {
                    if balance.total > Decimal::ZERO {
                        asset_balances.push((asset, balance));
                    }
                }
                Err(_) => continue, // Skip assets with errors
            }
        }
        
        Ok(asset_balances)
    }
    
    pub async fn get_system_balances(&self) -> Result<Vec<(Asset, Balance, Balance)>, WalletError> {
        // Get all assets
        let assets = self.asset_store.list_assets().await;
        
        // Get hot wallet and cold storage balances for each asset
        let mut system_balances = Vec::new();
        
        let hot_wallets = self.hot_wallets.read().await;
        let cold_storages = self.cold_storages.read().await;
        
        for asset in assets {
            // Get hot wallet balance
            let hot_wallet = match hot_wallets.get(&asset.id) {
                Some(wallet) => wallet,
                None => continue, // Skip assets without hot wallet
            };
            
            let hot_balance = match hot_wallet.get_hot_wallet_balance().await {
                Ok(balance) => balance,
                Err(_) => continue, // Skip assets with errors
            };
            
            // Get cold storage balance
            let cold_storage = match cold_storages.get(&asset.id) {
                Some(storage) => storage,
                None => continue, // Skip assets without cold storage
            };
            
            let cold_balance = match cold_storage.get_cold_storage_balance().await {
                Ok(balance) => balance,
                Err(_) => continue, // Skip assets with errors
            };
            
            system_balances.push((asset, hot_balance, cold_balance));
        }
        
        Ok(system_balances)
    }
}

// Tests for the wallet system
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[tokio::test]
    async fn test_wallet_system() {
        // Create wallet system
        let wallet_system = WalletSystem::new();
        
        // Create an asset
        let btc_asset = Asset {
            id: "BTC".to_string(),
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            blockchain_id: "bitcoin".to_string(),
            decimals: 8,
            is_active: true,
            min_deposit: dec!(0.001),
            min_withdrawal: dec!(0.001),
            withdrawal_fee: dec!(0.0005),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Create a blockchain adapter
        let blockchain_adapter = Arc::new(MockBlockchainAdapter::new("BTC".to_string(), dec!(0.0001)));
        
        // Register asset
        wallet_system.register_asset(btc_asset.clone(), blockchain_adapter).await.unwrap();
        
        // Initialize system wallets
        wallet_system
            .initialize_system_wallets(&"BTC".to_string(), dec!(10), dec!(50))
            .await
            .unwrap();
        
        // Create a user
        let user_id = Uuid::new_v4();
        let user_wallet = wallet_system.create_user_wallet(user_id).await.unwrap();
        
        // Generate deposit address
        let address_info = wallet_system
            .generate_deposit_address(&user_id, &"BTC".to_string())
            .await
            .unwrap();
        
        assert_eq!(address_info.wallet_id, user_wallet.id);
        assert_eq!(address_info.asset_id, "BTC");
        assert!(address_info.is_active);
        
        // For testing, we'll directly manipulate the balances to simulate a deposit
        let hot_wallets = wallet_system.hot_wallets.read().await;
        let btc_hot_wallet = hot_wallets.get("BTC").unwrap();
        
        // Simulate receiving a deposit
        let deposit_tx = btc_hot_wallet
            .process_deposit(&user_wallet.id, dec!(1.5), "mocktx123".to_string())
            .await
            .unwrap();
        
        assert_eq!(deposit_tx.status, TransactionStatus::Completed);
        assert_eq!(deposit_tx.amount, dec!(1.5));
        
        // Check user balance
        let user_balance = wallet_system.get_user_balance(&user_id, &"BTC".to_string()).await.unwrap();
        assert_eq!(user_balance.total, dec!(1.5));
        assert_eq!(user_balance.available, dec!(1.5));
        
        // Try a withdrawal
        let withdrawal_tx = wallet_system
            .process_withdrawal(&user_id, &"BTC".to_string(), "BTC-withdrawaladdr", dec!(0.5), None)
            .await
            .unwrap();
        
        assert_eq!(withdrawal_tx.status, TransactionStatus::Completed);
        assert_eq!(withdrawal_tx.amount, dec!(0.5));
        
        // Check updated user balance (1.5 - 0.5 - fee)
        let user_balance = wallet_system.get_user_balance(&user_id, &"BTC".to_string()).await.unwrap();
        assert!(user_balance.total < dec!(1.0)); // Should be less than 1.0 due to fees
        
        // Test transferring to cold storage
        let cold_tx = wallet_system
            .transfer_to_cold_storage(&"BTC".to_string(), dec!(0.3))
            .await
            .unwrap();
        
        assert_eq!(cold_tx.status, TransactionStatus::Completed);
        assert_eq!(cold_tx.transaction_type, TransactionType::Transfer);
        
        // Get system balances
        let system_balances = wallet_system.get_system_balances().await.unwrap();
        assert_eq!(system_balances.len(), 1);
        
        let (asset, hot_balance, cold_balance) = &system_balances[0];
        assert_eq!(asset.id, "BTC");
        assert!(cold_balance.total > Decimal::ZERO); // Cold storage should have balance now
    }
}
