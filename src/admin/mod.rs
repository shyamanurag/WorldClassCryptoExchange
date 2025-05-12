

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use async_trait::async_trait;

use crate::kyc::{UserProfile, UserStatus, IdentityTier, VerificationStatus, DocumentType, RiskLevel};
use crate::kyc::{UserDocument, VerificationAttempt, AmlCheckResult, AmlResult, AmlMatchStatus};
use crate::trading_engine::matching_engine::{Side, OrderType, Order, OrderStatus};

// Type definitions
pub type UserId = Uuid;
pub type AdminId = Uuid;
pub type AdminActionId = Uuid;
pub type AdminRoleId = Uuid;
pub type AdminTaskId = Uuid;
pub type SecurityIncidentId = Uuid;
pub type TradingPairId = String;
pub type ReportId = Uuid;

// Admin roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminRoleType {
    SuperAdmin,
    ComplianceAdmin,
    SecurityAdmin,
    CustomerSupportAdmin,
    TreasuryAdmin,
    SystemAdmin,
    AuditAdmin,
    ReadOnlyAdmin,
}

// Admin permission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminPermission {
    UserVerification,
    UserAccountManagement,
    UserSuspension,
    WithdrawalManagement,
    DepositManagement,
    TradingPairManagement,
    FeeManagement,
    SystemConfiguration,
    AccessLogs,
    SecurityAlerts,
    CustomerSupportTickets,
    TreasuryOperations,
    ColdWalletAccess,
    AuditLogAccess,
    ReportGeneration,
    AdminUserManagement,
}

// Admin action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminActionType {
    UserAccountApproval,
    UserAccountSuspension,
    UserAccountUnsuspension,
    UserAccountClosure,
    UserTierUpgrade,
    UserTierDowngrade,
    WithdrawalApproval,
    WithdrawalRejection,
    DepositManualCredit,
    DepositReversal,
    TradingPairAddition,
    TradingPairRemoval,
    TradingPairSuspension,
    TradingPairReactivation,
    FeeAdjustment,
    SystemParameterUpdate,
    SecurityAlertReview,
    SecurityIncidentCreation,
    CustomerSupportTicketResolution,
    AdminUserCreation,
    AdminUserSuspension,
    TreasuryOperation,
    ColdWalletTransfer,
    EmergencySystemShutdown,
    EmergencySystemRecovery,
    ReportGeneration,
    AuditLogReview,
}

// Admin action target types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminActionTargetType {
    User,
    Admin,
    TradingPair,
    Deposit,
    Withdrawal,
    Transaction,
    SystemParameter,
    SecurityAlert,
    CustomerSupportTicket,
    TreasuryWallet,
    ColdWallet,
    HotWallet,
    TradingEngine,
    WalletSystem,
    ApiGateway,
    Report,
    AuditLog,
}

// Admin profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminProfile {
    pub id: AdminId,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub password_hash: String,
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub roles: Vec<AdminRoleId>,
    pub permissions: Vec<AdminPermission>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub requires_password_change: bool,
    pub failed_login_attempts: i32,
    pub last_password_change: DateTime<Utc>,
    pub ip_whitelist: Option<Vec<String>>,
}

// Admin role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRole {
    pub id: AdminRoleId,
    pub name: String,
    pub role_type: AdminRoleType,
    pub permissions: Vec<AdminPermission>,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Admin action log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAction {
    pub id: AdminActionId,
    pub admin_id: AdminId,
    pub action_type: AdminActionType,
    pub target_type: AdminActionTargetType,
    pub target_id: String,
    pub details: serde_json::Value,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub status: AdminActionStatus,
    pub approval_required: bool,
    pub approver_id: Option<AdminId>,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

// Admin action status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminActionStatus {
    Pending,
    Approved,
    Rejected,
    Completed,
    Failed,
    Cancelled,
}

// Admin task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminTask {
    pub id: AdminTaskId,
    pub title: String,
    pub description: String,
    pub assigned_to: Option<AdminId>,
    pub created_by: AdminId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub related_action_id: Option<AdminActionId>,
    pub related_user_id: Option<UserId>,
    pub category: TaskCategory,
    pub tags: Vec<String>,
    pub completion_notes: Option<String>,
}

// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Open,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
}

// Task category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCategory {
    UserVerification,
    Compliance,
    CustomerSupport,
    Security,
    Treasury,
    TradingPairs,
    System,
    Other,
}

// Security incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub id: SecurityIncidentId,
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub reported_by: AdminId,
    pub assigned_to: Option<AdminId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
    pub affected_users: Vec<UserId>,
    pub affected_systems: Vec<String>,
    pub incident_type: IncidentType,
    pub indicators: Vec<String>,
    pub action_taken: Option<String>,
    pub related_actions: Vec<AdminActionId>,
}

// Incident severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Incident status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    New,
    Investigating,
    Mitigated,
    Resolved,
    Closed,
}

// Incident type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentType {
    Phishing,
    AccountCompromise,
    DataBreach,
    DDoSAttack,
    UnauthorizedAccess,
    SuspiciousActivity,
    SystemAnomaly,
    ComplianceViolation,
    Other,
}

// Trading pair status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradingPairStatus {
    Active,
    Inactive,
    Delisted,
    MaintenanceMode,
    ComingSoon,
}

// Trading pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub id: TradingPairId,
    pub base_asset: String,
    pub quote_asset: String,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub price_precision: u32,
    pub min_quantity: Decimal,
    pub max_quantity: Decimal,
    pub quantity_precision: u32,
    pub min_notional: Decimal,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub status: TradingPairStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub listing_date: Option<DateTime<Utc>>,
    pub delisting_date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_leveraged: bool,
    pub max_leverage: Option<Decimal>,
}

// Fee schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSchedule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tier_levels: Vec<FeeTier>,
    pub default_maker_fee: Decimal,
    pub default_taker_fee: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Fee tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeTier {
    pub tier_level: i32,
    pub min_30d_volume: Decimal,
    pub min_token_holdings: Option<Decimal>,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub withdrawal_fee_discount: Decimal,
    pub description: String,
}

// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    UserActivity,
    TradingVolume,
    TradeHistory,
    DepositWithdrawal,
    FeesCollected,
    ProfitLoss,
    UserAcquisition,
    UserRetention,
    TradingPairPerformance,
    AuditReport,
    ComplianceReport,
    IncidentReport,
    SystemPerformance,
    CustodialBalances,
    MarketLiquidity,
    Custom,
}

// Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: ReportId,
    pub title: String,
    pub description: String,
    pub report_type: ReportType,
    pub created_by: AdminId,
    pub created_at: DateTime<Utc>,
    pub time_period: ReportTimePeriod,
    pub parameters: serde_json::Value,
    pub data: serde_json::Value,
    pub file_url: Option<String>,
    pub file_type: Option<String>,
    pub is_scheduled: bool,
    pub schedule_frequency: Option<ScheduleFrequency>,
    pub last_generated: DateTime<Utc>,
    pub next_generation: Option<DateTime<Utc>>,
}

// Report time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTimePeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

// Schedule frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleFrequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Yearly,
}

// Admin profile store
pub struct AdminProfileStore {
    profiles: RwLock<Vec<AdminProfile>>,
}

impl AdminProfileStore {
    pub fn new() -> Self {
        AdminProfileStore {
            profiles: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_profile(&self, profile: AdminProfile) -> Result<AdminId> {
        let mut profiles = self.profiles.write().await;
        
        // Check if username or email already exists
        if profiles.iter().any(|p| p.username == profile.username) {
            return Err(anyhow!("Username already exists: {}", profile.username));
        }
        
        if profiles.iter().any(|p| p.email == profile.email) {
            return Err(anyhow!("Email already exists: {}", profile.email));
        }
        
        let admin_id = profile.id;
        profiles.push(profile);
        Ok(admin_id)
    }
    
    pub async fn get_profile(&self, admin_id: &AdminId) -> Result<AdminProfile> {
        let profiles = self.profiles.read().await;
        profiles
            .iter()
            .find(|p| p.id == *admin_id)
            .cloned()
            .ok_or_else(|| anyhow!("Admin profile not found: {}", admin_id))
    }
    
    pub async fn get_profile_by_username(&self, username: &str) -> Result<AdminProfile> {
        let profiles = self.profiles.read().await;
        profiles
            .iter()
            .find(|p| p.username == username)
            .cloned()
            .ok_or_else(|| anyhow!("Admin profile not found for username: {}", username))
    }
    
    pub async fn update_profile(&self, profile: AdminProfile) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        
        let index = profiles
            .iter()
            .position(|p| p.id == profile.id)
            .ok_or_else(|| anyhow!("Admin profile not found: {}", profile.id))?;
        
        profiles[index] = profile;
        Ok(())
    }
    
    pub async fn deactivate_profile(&self, admin_id: &AdminId) -> Result<AdminProfile> {
        let mut profiles = self.profiles.write().await;
        
        let profile = profiles
            .iter_mut()
            .find(|p| p.id == *admin_id)
            .ok_or_else(|| anyhow!("Admin profile not found: {}", admin_id))?;
        
        profile.is_active = false;
        profile.updated_at = Utc::now();
        
        Ok(profile.clone())
    }
    
    pub async fn get_all_active_profiles(&self) -> Result<Vec<AdminProfile>> {
        let profiles = self.profiles.read().await;
        Ok(profiles.iter().filter(|p| p.is_active).cloned().collect())
    }
    
    pub async fn update_last_login(&self, admin_id: &AdminId, ip_address: &str, user_agent: &str) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        
        let profile = profiles
            .iter_mut()
            .find(|p| p.id == *admin_id)
            .ok_or_else(|| anyhow!("Admin profile not found: {}", admin_id))?;
        
        profile.last_login = Some(Utc::now());
        profile.failed_login_attempts = 0;
        
        Ok(())
    }
    
    pub async fn increment_failed_login(&self, username: &str) -> Result<i32> {
        let mut profiles = self.profiles.write().await;
        
        let profile = profiles
            .iter_mut()
            .find(|p| p.username == username)
            .ok_or_else(|| anyhow!("Admin profile not found for username: {}", username))?;
        
        profile.failed_login_attempts += 1;
        
        Ok(profile.failed_login_attempts)
    }
}

// Admin role store
pub struct AdminRoleStore {
    roles: RwLock<Vec<AdminRole>>,
}

impl AdminRoleStore {
    pub fn new() -> Self {
        AdminRoleStore {
            roles: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_role(&self, role: AdminRole) -> Result<AdminRoleId> {
        let mut roles = self.roles.write().await;
        
        // Check if role name already exists
        if roles.iter().any(|r| r.name == role.name) {
            return Err(anyhow!("Role name already exists: {}", role.name));
        }
        
        let role_id = role.id;
        roles.push(role);
        Ok(role_id)
    }
    
    pub async fn get_role(&self, role_id: &AdminRoleId) -> Result<AdminRole> {
        let roles = self.roles.read().await;
        roles
            .iter()
            .find(|r| r.id == *role_id)
            .cloned()
            .ok_or_else(|| anyhow!("Admin role not found: {}", role_id))
    }
    
    pub async fn get_roles_by_ids(&self, role_ids: &[AdminRoleId]) -> Result<Vec<AdminRole>> {
        let roles = self.roles.read().await;
        let result = roles
            .iter()
            .filter(|r| role_ids.contains(&r.id))
            .cloned()
            .collect();
        Ok(result)
    }
    
    pub async fn update_role(&self, role: AdminRole) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        let index = roles
            .iter()
            .position(|r| r.id == role.id)
            .ok_or_else(|| anyhow!("Admin role not found: {}", role.id))?;
        
        roles[index] = role;
        Ok(())
    }
    
    pub async fn get_all_roles(&self) -> Result<Vec<AdminRole>> {
        let roles = self.roles.read().await;
        Ok(roles.clone())
    }
    
    pub async fn initialize_default_roles(&self) -> Result<()> {
        let super_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "Super Admin".to_string(),
            role_type: AdminRoleType::SuperAdmin,
            permissions: vec![
                AdminPermission::UserVerification,
                AdminPermission::UserAccountManagement,
                AdminPermission::UserSuspension,
                AdminPermission::WithdrawalManagement,
                AdminPermission::DepositManagement,
                AdminPermission::TradingPairManagement,
                AdminPermission::FeeManagement,
                AdminPermission::SystemConfiguration,
                AdminPermission::AccessLogs,
                AdminPermission::SecurityAlerts,
                AdminPermission::CustomerSupportTickets,
                AdminPermission::TreasuryOperations,
                AdminPermission::ColdWalletAccess,
                AdminPermission::AuditLogAccess,
                AdminPermission::ReportGeneration,
                AdminPermission::AdminUserManagement,
            ],
            description: "Full access to all systems and functionalities".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let compliance_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "Compliance Admin".to_string(),
            role_type: AdminRoleType::ComplianceAdmin,
            permissions: vec![
                AdminPermission::UserVerification,
                AdminPermission::UserAccountManagement,
                AdminPermission::AccessLogs,
                AdminPermission::AuditLogAccess,
                AdminPermission::ReportGeneration,
            ],
            description: "Access to user verification and compliance-related functions".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let security_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "Security Admin".to_string(),
            role_type: AdminRoleType::SecurityAdmin,
            permissions: vec![
                AdminPermission::UserSuspension,
                AdminPermission::AccessLogs,
                AdminPermission::SecurityAlerts,
                AdminPermission::AuditLogAccess,
            ],
            description: "Access to security-related functions and alerts".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let customer_support_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "Customer Support Admin".to_string(),
            role_type: AdminRoleType::CustomerSupportAdmin,
            permissions: vec![
                AdminPermission::UserAccountManagement,
                AdminPermission::CustomerSupportTickets,
            ],
            description: "Access to customer support and user management functions".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let treasury_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "Treasury Admin".to_string(),
            role_type: AdminRoleType::TreasuryAdmin,
            permissions: vec![
                AdminPermission::WithdrawalManagement,
                AdminPermission::DepositManagement,
                AdminPermission::TreasuryOperations,
                AdminPermission::ColdWalletAccess,
            ],
            description: "Access to treasury and wallet operations".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let system_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "System Admin".to_string(),
            role_type: AdminRoleType::SystemAdmin,
            permissions: vec![
                AdminPermission::TradingPairManagement,
                AdminPermission::FeeManagement,
                AdminPermission::SystemConfiguration,
            ],
            description: "Access to system configuration and trading pair management".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let audit_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "Audit Admin".to_string(),
            role_type: AdminRoleType::AuditAdmin,
            permissions: vec![
                AdminPermission::AccessLogs,
                AdminPermission::AuditLogAccess,
                AdminPermission::ReportGeneration,
            ],
            description: "Access to audit logs and reporting".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let read_only_admin = AdminRole {
            id: Uuid::new_v4(),
            name: "Read-Only Admin".to_string(),
            role_type: AdminRoleType::ReadOnlyAdmin,
            permissions: vec![
                AdminPermission::AccessLogs,
                AdminPermission::ReportGeneration,
            ],
            description: "Read-only access to logs and reports".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.add_role(super_admin).await?;
        self.add_role(compliance_admin).await?;
        self.add_role(security_admin).await?;
        self.add_role(customer_support_admin).await?;
        self.add_role(treasury_admin).await?;
        self.add_role(system_admin).await?;
        self.add_role(audit_admin).await?;
        self.add_role(read_only_admin).await?;
        
        Ok(())
    }
}

// Admin action log store
pub struct AdminActionStore {
    actions: RwLock<Vec<AdminAction>>,
}

impl AdminActionStore {
    pub fn new() -> Self {
        AdminActionStore {
            actions: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_action(&self, action: AdminAction) -> Result<AdminActionId> {
        let mut actions = self.actions.write().await;
        let action_id = action.id;
        actions.push(action);
        Ok(action_id)
    }
    
    pub async fn get_action(&self, action_id: &AdminActionId) -> Result<AdminAction> {
        let actions = self.actions.read().await;
        actions
            .iter()
            .find(|a| a.id == *action_id)
            .cloned()
            .ok_or_else(|| anyhow!("Admin action not found: {}", action_id))
    }
    
    pub async fn update_action_status(&self, action_id: &AdminActionId, status: AdminActionStatus, notes: Option<String>) -> Result<AdminAction> {
        let mut actions = self.actions.write().await;
        
        let action = actions
            .iter_mut()
            .find(|a| a.id == *action_id)
            .ok_or_else(|| anyhow!("Admin action not found: {}", action_id))?;
        
        action.status = status;
        
        if let Some(n) = notes {
            action.notes = Some(n);
        }
        
        Ok(action.clone())
    }
    
    pub async fn approve_action(&self, action_id: &AdminActionId, approver_id: &AdminId, notes: Option<String>) -> Result<AdminAction> {
        let mut actions = self.actions.write().await;
        
        let action = actions
            .iter_mut()
            .find(|a| a.id == *action_id)
            .ok_or_else(|| anyhow!("Admin action not found: {}", action_id))?;
        
        if !action.approval_required {
            return Err(anyhow!("Action does not require approval"));
        }
        
        if action.status != AdminActionStatus::Pending {
            return Err(anyhow!("Action is not in pending status"));
        }
        
        action.status = AdminActionStatus::Approved;
        action.approver_id = Some(*approver_id);
        action.approved_at = Some(Utc::now());
        
        if let Some(n) = notes {
            action.notes = Some(n);
        }
        
        Ok(action.clone())
    }
    
    pub async fn get_pending_actions(&self) -> Result<Vec<AdminAction>> {
        let actions = self.actions.read().await;
        Ok(actions
            .iter()
            .filter(|a| a.status == AdminActionStatus::Pending && a.approval_required)
            .cloned()
            .collect())
    }
    
    pub async fn get_admin_actions(&self, admin_id: &AdminId) -> Result<Vec<AdminAction>> {
        let actions = self.actions.read().await;
        Ok(actions
            .iter()
            .filter(|a| a.admin_id == *admin_id)
            .cloned()
            .collect())
    }
    
    pub async fn get_actions_by_target(&self, target_type: AdminActionTargetType, target_id: &str) -> Result<Vec<AdminAction>> {
        let actions = self.actions.read().await;
        Ok(actions
            .iter()
            .filter(|a| a.target_type == target_type && a.target_id == target_id)
            .cloned()
            .collect())
    }
    
    pub async fn get_recent_actions(&self, limit: usize) -> Result<Vec<AdminAction>> {
        let actions = self.actions.read().await;
        let mut recent_actions = actions.clone();
        recent_actions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(recent_actions.into_iter().take(limit).collect())
    }
}

// Admin task store
pub struct AdminTaskStore {
    tasks: RwLock<Vec<AdminTask>>,
}

impl AdminTaskStore {
    pub fn new() -> Self {
        AdminTaskStore {
            tasks: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_task(&self, task: AdminTask) -> Result<AdminTaskId> {
        let mut tasks = self.tasks.write().await;
        let task_id = task.id;
        tasks.push(task);
        Ok(task_id)
    }
    
    pub async fn get_task(&self, task_id: &AdminTaskId) -> Result<AdminTask> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .find(|t| t.id == *task_id)
            .cloned()
            .ok_or_else(|| anyhow!("Admin task not found: {}", task_id))
    }
    
    pub async fn update_task(&self, task: AdminTask) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        
        let index = tasks
            .iter()
            .position(|t| t.id == task.id)
            .ok_or_else(|| anyhow!("Admin task not found: {}", task.id))?;
        
        tasks[index] = task;
        Ok(())
    }
    
    pub async fn update_task_status(&self, task_id: &AdminTaskId, status: TaskStatus, completion_notes: Option<String>) -> Result<AdminTask> {
        let mut tasks = self.tasks.write().await;
        
        let task = tasks
            .iter_mut()
            .find(|t| t.id == *task_id)
            .ok_or_else(|| anyhow!("Admin task not found: {}", task_id))?;
        
        task.status = status;
        task.updated_at = Utc::now();
        
        if let Some(notes) = completion_notes {
            task.completion_notes = Some(notes);
        }
        
        Ok(task.clone())
    }
    
    pub async fn assign_task(&self, task_id: &AdminTaskId, admin_id: &AdminId) -> Result<AdminTask> {
        let mut tasks = self.tasks.write().await;
        
        let task = tasks
            .iter_mut()
            .find(|t| t.id == *task_id)
            .ok_or_else(|| anyhow!("Admin task not found: {}", task_id))?;
        
        task.assigned_to = Some(*admin_id);
        task.updated_at = Utc::now();
        
        Ok(task.clone())
    }
    
    pub async fn get_admin_tasks(&self, admin_id: &AdminId, status: Option<TaskStatus>) -> Result<Vec<AdminTask>> {
        let tasks = self.tasks.read().await;
        let filtered = tasks
            .iter()
            .filter(|t| t.assigned_to == Some(*admin_id))
            .filter(|t| status.map_or(true, |s| t.status == s))
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    pub async fn get_pending_tasks(&self) -> Result<Vec<AdminTask>> {
        let tasks = self.tasks.read().await;
        Ok(tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Open || t.status == TaskStatus::InProgress)
            .cloned()
            .collect())
    }
    
    pub async fn get_overdue_tasks(&self) -> Result<Vec<AdminTask>> {
        let now = Utc::now();
        let tasks = self.tasks.read().await;
        Ok(tasks
            .iter()
            .filter(|t| {
                t.due_date.map_or(false, |d| d < now) && 
                (t.status == TaskStatus::Open || t.status == TaskStatus::InProgress)
            })
            .cloned()
            .collect())
    }
}

// Security incident store
pub struct SecurityIncidentStore {
    incidents: RwLock<Vec<SecurityIncident>>,
}

impl SecurityIncidentStore {
    pub fn new() -> Self {
        SecurityIncidentStore {
            incidents: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_incident(&self, incident: SecurityIncident) -> Result<SecurityIncidentId> {
        let mut incidents = self.incidents.write().await;
        let incident_id = incident.id;
        incidents.push(incident);
        Ok(incident_id)
    }
    
    pub async fn get_incident(&self, incident_id: &SecurityIncidentId) -> Result<SecurityIncident> {
        let incidents = self.incidents.read().await;
        incidents
            .iter()
            .find(|i| i.id == *incident_id)
            .cloned()
            .ok_or_else(|| anyhow!("Security incident not found: {}", incident_id))
    }
    
    pub async fn update_incident(&self, incident: SecurityIncident) -> Result<()> {
        let mut incidents = self.incidents.write().await;
        
        let index = incidents
            .iter()
            .position(|i| i.id == incident.id)
            .ok_or_else(|| anyhow!("Security incident not found: {}", incident.id))?;
        
        incidents[index] = incident;
        Ok(())
    }
    
    pub async fn update_incident_status(&self, incident_id: &SecurityIncidentId, status: IncidentStatus, notes: Option<String>) -> Result<SecurityIncident> {
        let mut incidents = self.incidents.write().await;
        
        let incident = incidents
            .iter_mut()
            .find(|i| i.id == *incident_id)
            .ok_or_else(|| anyhow!("Security incident not found: {}", incident_id))?;
        
        incident.status = status;
        incident.updated_at = Utc::now();
        
        if status == IncidentStatus::Resolved || status == IncidentStatus::Closed {
            incident.resolved_at = Some(Utc::now());
        }
        
        if let Some(n) = notes {
            incident.resolution_notes = Some(n);
        }
        
        Ok(incident.clone())
    }
    
    pub async fn assign_incident(&self, incident_id: &SecurityIncidentId, admin_id: &AdminId) -> Result<SecurityIncident> {
        let mut incidents = self.incidents.write().await;
        
        let incident = incidents
            .iter_mut()
            .find(|i| i.id == *incident_id)
            .ok_or_else(|| anyhow!("Security incident not found: {}", incident_id))?;
        
        incident.assigned_to = Some(*admin_id);
        incident.updated_at = Utc::now();
        
        Ok(incident.clone())
    }
    
    pub async fn get_open_incidents(&self) -> Result<Vec<SecurityIncident>> {
        let incidents = self.incidents.read().await;
        Ok(incidents
            .iter()
            .filter(|i| i.status != IncidentStatus::Closed && i.status != IncidentStatus::Resolved)
            .cloned()
            .collect())
    }
    
    pub async fn get_critical_incidents(&self) -> Result<Vec<SecurityIncident>> {
        let incidents = self.incidents.read().await;
        Ok(incidents
            .iter()
            .filter(|i| i.severity == IncidentSeverity::Critical && i.status != IncidentStatus::Closed)
            .cloned()
            .collect())
    }
    
    pub async fn get_incidents_by_type(&self, incident_type: IncidentType) -> Result<Vec<SecurityIncident>> {
        let incidents = self.incidents.read().await;
        Ok(incidents
            .iter()
            .filter(|i| i.incident_type == incident_type)
            .cloned()
            .collect())
    }
    
    pub async fn get_incidents_affecting_user(&self, user_id: &UserId) -> Result<Vec<SecurityIncident>> {
        let incidents = self.incidents.read().await;
        Ok(incidents
            .iter()
            .filter(|i| i.affected_users.contains(user_id))
            .cloned()
            .collect())
    }
}

// Trading pair store
pub struct TradingPairStore {
    pairs: RwLock<Vec<TradingPair>>,
}

impl TradingPairStore {
    pub fn new() -> Self {
        TradingPairStore {
            pairs: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_pair(&self, pair: TradingPair) -> Result<TradingPairId> {
        let mut pairs = self.pairs.write().await;
        
        // Check if pair already exists
        if pairs.iter().any(|p| p.id == pair.id) {
            return Err(anyhow!("Trading pair already exists: {}", pair.id));
        }
        
        let pair_id = pair.id.clone();
        pairs.push(pair);
        Ok(pair_id)
    }
    
    pub async fn get_pair(&self, pair_id: &TradingPairId) -> Result<TradingPair> {
        let pairs = self.pairs.read().await;
        pairs
            .iter()
            .find(|p| p.id == *pair_id)
            .cloned()
            .ok_or_else(|| anyhow!("Trading pair not found: {}", pair_id))
    }
    
    pub async fn update_pair(&self, pair: TradingPair) -> Result<()> {
        let mut pairs = self.pairs.write().await;
        
        let index = pairs
            .iter()
            .position(|p| p.id == pair.id)
            .ok_or_else(|| anyhow!("Trading pair not found: {}", pair.id))?;
        
        pairs[index] = pair;
        Ok(())
    }
    
    pub async fn update_pair_status(&self, pair_id: &TradingPairId, status: TradingPairStatus) -> Result<TradingPair> {
        let mut pairs = self.pairs.write().await;
        
        let pair = pairs
            .iter_mut()
            .find(|p| p.id == *pair_id)
            .ok_or_else(|| anyhow!("Trading pair not found: {}", pair_id))?;
        
        pair.status = status;
        pair.updated_at = Utc::now();
        
        if status == TradingPairStatus::Delisted {
            pair.delisting_date = Some(Utc::now());
        }
        
        Ok(pair.clone())
    }
    
    pub async fn get_all_pairs(&self) -> Result<Vec<TradingPair>> {
        let pairs = self.pairs.read().await;
        Ok(pairs.clone())
    }
    
    pub async fn get_active_pairs(&self) -> Result<Vec<TradingPair>> {
        let pairs = self.pairs.read().await;
        Ok(pairs
            .iter()
            .filter(|p| p.status == TradingPairStatus::Active)
            .cloned()
            .collect())
    }
    
    pub async fn get_pairs_by_base_asset(&self, base_asset: &str) -> Result<Vec<TradingPair>> {
        let pairs = self.pairs.read().await;
        Ok(pairs
            .iter()
            .filter(|p| p.base
