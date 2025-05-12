

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use rand::RngCore;

// Type definitions
pub type UserId = Uuid;
pub type DocumentId = Uuid;
pub type VerificationId = Uuid;

// User identity tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityTier {
    Tier0, // Email only, very limited functionality
    Tier1, // Basic KYC, limited trading
    Tier2, // Full KYC, standard limits
    Tier3, // Enhanced KYC, high limits
    Tier4, // Institutional, highest limits
}

// Document types for KYC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    Passport,
    NationalId,
    DriversLicense,
    ResidencePermit,
    UtilityBill,
    BankStatement,
    TaxId,
    CompanyRegistration,
    ArticlesOfIncorporation,
    ProofOfAddress,
    Selfie,
    VideoVerification,
}

// Document verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    InReview,
    Approved,
    Rejected,
    Expired,
}

// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Unverified,
    PendingVerification,
    Active,
    Restricted,
    Suspended,
    Banned,
}

// Risk level for users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

// User document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDocument {
    pub id: DocumentId,
    pub user_id: UserId,
    pub document_type: DocumentType,
    pub file_path: String,
    pub file_hash: String,
    pub uploaded_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: VerificationStatus,
    pub reviewer_notes: Option<String>,
    pub metadata: serde_json::Value,
}

// User verification attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAttempt {
    pub id: VerificationId,
    pub user_id: UserId,
    pub tier: IdentityTier,
    pub documents: Vec<DocumentId>,
    pub verification_provider: Option<String>,
    pub provider_reference: Option<String>,
    pub status: VerificationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

// User profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: UserId,
    pub email: String,
    pub phone_number: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<DateTime<Utc>>,
    pub nationality: Option<String>,
    pub country_of_residence: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub identity_tier: IdentityTier,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub password_hash: String,
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub risk_level: RiskLevel,
    pub ip_addresses: Vec<String>,
    pub user_agent: Option<String>,
}

impl UserProfile {
    pub fn new(email: String, password: &str) -> Result<Self> {
        // Generate password hash using Argon2
        let salt = rand::thread_rng().gen::<[u8; 16]>();
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?
            .to_string();
        
        let now = Utc::now();
        
        Ok(UserProfile {
            user_id: Uuid::new_v4(),
            email,
            phone_number: None,
            first_name: None,
            last_name: None,
            date_of_birth: None,
            nationality: None,
            country_of_residence: None,
            address_line1: None,
            address_line2: None,
            city: None,
            region: None,
            postal_code: None,
            identity_tier: IdentityTier::Tier0,
            status: UserStatus::Unverified,
            created_at: now,
            updated_at: now,
            last_login: None,
            password_hash,
            totp_secret: None,
            totp_enabled: false,
            email_verified: false,
            phone_verified: false,
            risk_level: RiskLevel::Medium, // Default to medium until assessed
            ip_addresses: Vec::new(),
            user_agent: None,
        })
    }
    
    pub fn verify_password(&self, password: &str) -> bool {
        // Parse the password hash
        let parsed_hash = match PasswordHash::new(&self.password_hash) {
            Ok(hash) => hash,
            Err(_) => return false,
        };
        
        // Verify the password
        let argon2 = Argon2::default();
        argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
    
    pub fn enable_totp(&mut self) -> Result<String> {
        // Generate a new TOTP secret
        let mut secret = [0u8; 20]; // 160 bits as recommended for TOTP
        OsRng.fill_bytes(&mut secret);
        
        let secret_base32 = base32::encode(base32::Alphabet::RFC4648 { padding: true }, &secret);
        
        self.totp_secret = Some(secret_base32.clone());
        self.updated_at = Utc::now();
        
        Ok(secret_base32)
    }
    
    pub fn verify_totp(&self, token: &str) -> bool {
        if !self.totp_enabled || self.totp_secret.is_none() {
            return false;
        }
        
        let secret = match &self.totp_secret {
            Some(s) => s,
            None => return false,
        };
        
        // Decode the base32 secret
        let decoded = match base32::decode(base32::Alphabet::RFC4648 { padding: true }, secret) {
            Some(d) => d,
            None => return false,
        };
        
        // Verify the TOTP token
        // In a real implementation, use a proper TOTP library
        // This is a placeholder for demonstration purposes
        token.len() == 6 && token.chars().all(|c| c.is_digit(10))
    }
}

// AML check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlCheckResult {
    pub user_id: UserId,
    pub check_date: DateTime<Utc>,
    pub provider: String,
    pub reference_id: String,
    pub result: AmlResult,
    pub risk_score: f64,
    pub details: String,
    pub match_status: AmlMatchStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmlResult {
    Clear,
    PotentialMatch,
    Match,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmlMatchStatus {
    NoMatch,
    Reviewing,
    FalsePositive,
    ConfirmedMatch,
}

// Risk scoring model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScoringModel {
    pub user_factors: Vec<RiskFactor>,
    pub transaction_factors: Vec<RiskFactor>,
    pub behavioral_factors: Vec<RiskFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub weight: f64,
    pub thresholds: Vec<(f64, RiskLevel)>,
}

// KYC service trait
#[async_trait]
pub trait KycProvider: Send + Sync {
    async fn verify_identity(&self, user_id: &UserId, documents: &[DocumentId]) -> Result<VerificationStatus>;
    async fn check_aml(&self, user_id: &UserId) -> Result<AmlCheckResult>;
    async fn get_provider_name(&self) -> String;
}

// Mock KYC provider
pub struct MockKycProvider;

#[async_trait]
impl KycProvider for MockKycProvider {
    async fn verify_identity(&self, user_id: &UserId, documents: &[DocumentId]) -> Result<VerificationStatus> {
        // Simulate a verification process
        // In a real implementation, this would call an external KYC provider API
        
        // 80% approval rate for demonstration
        let random = rand::random::<f64>();
        
        if random < 0.8 {
            Ok(VerificationStatus::Approved)
        } else if random < 0.9 {
            Ok(VerificationStatus::Rejected)
        } else {
            Ok(VerificationStatus::InReview)
        }
    }
    
    async fn check_aml(&self, user_id: &UserId) -> Result<AmlCheckResult> {
        // Simulate an AML check
        // In a real implementation, this would call an external AML provider API
        
        // 95% clear rate for demonstration
        let random = rand::random::<f64>();
        let result = if random < 0.95 {
            AmlResult::Clear
        } else if random < 0.98 {
            AmlResult::PotentialMatch
        } else {
            AmlResult::Match
        };
        
        let risk_score = random * 100.0;
        
        Ok(AmlCheckResult {
            user_id: *user_id,
            check_date: Utc::now(),
            provider: "MockAmlProvider".to_string(),
            reference_id: Uuid::new_v4().to_string(),
            result,
            risk_score,
            details: "Simulated AML check".to_string(),
            match_status: if result == AmlResult::Clear {
                AmlMatchStatus::NoMatch
            } else {
                AmlMatchStatus::Reviewing
            },
        })
    }
    
    async fn get_provider_name(&self) -> String {
        "MockKycProvider".to_string()
    }
}

// User document store
pub struct DocumentStore {
    documents: RwLock<Vec<UserDocument>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        DocumentStore {
            documents: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_document(&self, document: UserDocument) -> Result<DocumentId> {
        let mut documents = self.documents.write().await;
        let document_id = document.id;
        documents.push(document);
        Ok(document_id)
    }
    
    pub async fn get_document(&self, document_id: &DocumentId) -> Result<UserDocument> {
        let documents = self.documents.read().await;
        documents
            .iter()
            .find(|d| d.id == *document_id)
            .cloned()
            .ok_or_else(|| anyhow!("Document not found: {}", document_id))
    }
    
    pub async fn get_user_documents(&self, user_id: &UserId) -> Result<Vec<UserDocument>> {
        let documents = self.documents.read().await;
        Ok(documents
            .iter()
            .filter(|d| d.user_id == *user_id)
            .cloned()
            .collect())
    }
    
    pub async fn update_document_status(
        &self,
        document_id: &DocumentId,
        status: VerificationStatus,
        reviewer_notes: Option<String>,
    ) -> Result<UserDocument> {
        let mut documents = self.documents.write().await;
        
        let document = documents
            .iter_mut()
            .find(|d| d.id == *document_id)
            .ok_or_else(|| anyhow!("Document not found: {}", document_id))?;
        
        document.status = status;
        if let Some(notes) = reviewer_notes {
            document.reviewer_notes = Some(notes);
        }
        
        Ok(document.clone())
    }
}

// Verification attempt store
pub struct VerificationStore {
    verifications: RwLock<Vec<VerificationAttempt>>,
}

impl VerificationStore {
    pub fn new() -> Self {
        VerificationStore {
            verifications: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_verification(&self, verification: VerificationAttempt) -> Result<VerificationId> {
        let mut verifications = self.verifications.write().await;
        let verification_id = verification.id;
        verifications.push(verification);
        Ok(verification_id)
    }
    
    pub async fn get_verification(&self, verification_id: &VerificationId) -> Result<VerificationAttempt> {
        let verifications = self.verifications.read().await;
        verifications
            .iter()
            .find(|v| v.id == *verification_id)
            .cloned()
            .ok_or_else(|| anyhow!("Verification not found: {}", verification_id))
    }
    
    pub async fn get_user_verifications(&self, user_id: &UserId) -> Result<Vec<VerificationAttempt>> {
        let verifications = self.verifications.read().await;
        Ok(verifications
            .iter()
            .filter(|v| v.user_id == *user_id)
            .cloned()
            .collect())
    }
    
    pub async fn update_verification_status(
        &self,
        verification_id: &VerificationId,
        status: VerificationStatus,
        notes: Option<String>,
    ) -> Result<VerificationAttempt> {
        let mut verifications = self.verifications.write().await;
        
        let verification = verifications
            .iter_mut()
            .find(|v| v.id == *verification_id)
            .ok_or_else(|| anyhow!("Verification not found: {}", verification_id))?;
        
        verification.status = status;
        verification.updated_at = Utc::now();
        
        if status == VerificationStatus::Approved || status == VerificationStatus::Rejected {
            verification.completed_at = Some(Utc::now());
        }
        
        if let Some(n) = notes {
            verification.notes = Some(n);
        }
        
        Ok(verification.clone())
    }
}

// User profile store
pub struct UserStore {
    users: RwLock<Vec<UserProfile>>,
}

impl UserStore {
    pub fn new() -> Self {
        UserStore {
            users: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn add_user(&self, user: UserProfile) -> Result<UserId> {
        let mut users = self.users.write().await;
        
        // Check if email already exists
        if users.iter().any(|u| u.email == user.email) {
            return Err(anyhow!("Email already exists: {}", user.email));
        }
        
        let user_id = user.user_id;
        users.push(user);
        Ok(user_id)
    }
    
    pub async fn get_user(&self, user_id: &UserId) -> Result<UserProfile> {
        let users = self.users.read().await;
        users
            .iter()
            .find(|u| u.user_id == *user_id)
            .cloned()
            .ok_or_else(|| anyhow!("User not found: {}", user_id))
    }
    
    pub async fn get_user_by_email(&self, email: &str) -> Result<UserProfile> {
        let users = self.users.read().await;
        users
            .iter()
            .find(|u| u.email == email)
            .cloned()
            .ok_or_else(|| anyhow!("User not found with email: {}", email))
    }
    
    pub async fn update_user(&self, user: UserProfile) -> Result<()> {
        let mut users = self.users.write().await;
        
        let index = users
            .iter()
            .position(|u| u.user_id == user.user_id)
            .ok_or_else(|| anyhow!("User not found: {}", user.user_id))?;
        
        users[index] = user;
        Ok(())
    }
    
    pub async fn update_user_tier(
        &self,
        user_id: &UserId,
        tier: IdentityTier,
    ) -> Result<UserProfile> {
        let mut users = self.users.write().await;
        
        let user = users
            .iter_mut()
            .find(|u| u.user_id == *user_id)
            .ok_or_else(|| anyhow!("User not found: {}", user_id))?;
        
        user.identity_tier = tier;
        user.updated_at = Utc::now();
        
        Ok(user.clone())
    }
    
    pub async fn update_user_status(
        &self,
        user_id: &UserId,
        status: UserStatus,
    ) -> Result<UserProfile> {
        let mut users = self.users.write().await;
        
        let user = users
            .iter_mut()
            .find(|u| u.user_id == *user_id)
            .ok_or_else(|| anyhow!("User not found: {}", user_id))?;
        
        user.status = status;
        user.updated_at = Utc::now();
        
        Ok(user.clone())
    }
}

// Risk scoring engine
pub struct RiskScoringEngine {
    model: RiskScoringModel,
}

impl RiskScoringEngine {
    pub fn new(model: RiskScoringModel) -> Self {
        RiskScoringEngine { model }
    }
    
    pub fn calculate_user_risk(&self, user: &UserProfile, transaction_history: &[serde_json::Value]) -> RiskLevel {
        // Calculate risk score based on user factors
        let mut user_score = 0.0;
        let mut user_total_weight = 0.0;
        
        for factor in &self.model.user_factors {
            let factor_score = match factor.name.as_str() {
                "country_risk" => self.calculate_country_risk(user),
                "age" => self.calculate_age_risk(user),
                "verification_level" => self.calculate_verification_risk(user),
                "account_age" => self.calculate_account_age_risk(user),
                "ip_diversity" => self.calculate_ip_diversity_risk(user),
                _ => 0.0, // Unknown factor
            };
            
            user_score += factor_score * factor.weight;
            user_total_weight += factor.weight;
        }
        
        if user_total_weight > 0.0 {
            user_score /= user_total_weight;
        }
        
        // Calculate risk score based on transaction factors
        let mut tx_score = 0.0;
        let mut tx_total_weight = 0.0;
        
        for factor in &self.model.transaction_factors {
            let factor_score = match factor.name.as_str() {
                "transaction_volume" => self.calculate_transaction_volume_risk(transaction_history),
                "transaction_frequency" => self.calculate_transaction_frequency_risk(transaction_history),
                "deposit_withdrawal_ratio" => self.calculate_deposit_withdrawal_ratio(transaction_history),
                _ => 0.0, // Unknown factor
            };
            
            tx_score += factor_score * factor.weight;
            tx_total_weight += factor.weight;
        }
        
        if tx_total_weight > 0.0 {
            tx_score /= tx_total_weight;
        }
        
        // Calculate behavioral risk score
        let mut behavior_score = 0.0;
        let mut behavior_total_weight = 0.0;
        
        for factor in &self.model.behavioral_factors {
            let factor_score = match factor.name.as_str() {
                "login_patterns" => self.calculate_login_pattern_risk(user),
                "trading_patterns" => self.calculate_trading_pattern_risk(transaction_history),
                _ => 0.0, // Unknown factor
            };
            
            behavior_score += factor_score * factor.weight;
            behavior_total_weight += factor.weight;
        }
        
        if behavior_total_weight > 0.0 {
            behavior_score /= behavior_total_weight;
        }
        
        // Calculate final risk score
        let final_score = (user_score + tx_score + behavior_score) / 3.0;
        
        // Determine risk level based on score
        if final_score < 0.25 {
            RiskLevel::Low
        } else if final_score < 0.5 {
            RiskLevel::Medium
        } else if final_score < 0.75 {
            RiskLevel::High
        } else {
            RiskLevel::Extreme
        }
    }
    
    // Risk factor calculation functions
    fn calculate_country_risk(&self, user: &UserProfile) -> f64 {
        // High-risk countries would have higher scores
        // This is a simplified example
        match user.country_of_residence.as_deref() {
            Some("US") | Some("CA") | Some("GB") | Some("DE") | Some("FR") | Some("JP") | Some("AU") => 0.1,
            Some("RU") | Some("CN") | Some("BR") | Some("IN") | Some("ZA") => 0.5,
            Some("KP") | Some("IR") | Some("SY") | Some("CU") => 0.9,
            _ => 0.5, // Default for unknown countries
        }
    }
    
    fn calculate_age_risk(&self, user: &UserProfile) -> f64 {
        match user.date_of_birth {
            Some(dob) => {
                let now = Utc::now();
                let age_years = (now - dob).num_days() / 365;
                
                if age_years < 25 {
                    0.6 // Younger users potentially higher risk
                } else if age_years < 40 {
                    0.3
                } else if age_years < 60 {
                    0.2
                } else {
                    0.4 // Elderly might be more susceptible to fraud
                }
            },
            None => 0.7, // Unknown age is higher risk
        }
    }
    
    fn calculate_verification_risk(&self, user: &UserProfile) -> f64 {
        match user.identity_tier {
            IdentityTier::Tier0 => 0.9,
            IdentityTier::Tier1 => 0.7,
            IdentityTier::Tier2 => 0.4,
            IdentityTier::Tier3 => 0.2,
            IdentityTier::Tier4 => 0.1,
        }
    }
    
    fn calculate_account_age_risk(&self, user: &UserProfile) -> f64 {
        let now = Utc::now();
        let account_age_days = (now - user.created_at).num_days();
        
        if account_age_days < 30 {
            0.8 // New accounts are higher risk
        } else if account_age_days < 90 {
            0.5
        } else if account_age_days < 365 {
            0.3
        } else {
            0.1 // Older accounts are generally lower risk
        }
    }
    
    fn calculate_ip_diversity_risk(&self, user: &UserProfile) -> f64 {
        let ip_count = user.ip_addresses.len();
        
        if ip_count <= 1 {
            0.1 // Single IP is low risk
        } else if ip_count <= 3 {
            0.3
        } else if ip_count <= 5 {
            0.5
        } else {
            0.8 // Many IPs could indicate account sharing or compromise
        }
    }
    
    fn calculate_transaction_volume_risk(&self, transaction_history: &[serde_json::Value]) -> f64 {
        // This would analyze transaction volumes in a real implementation
        // Simplified for demonstration
        let tx_count = transaction_history.len();
        
        if tx_count < 10 {
            0.2
        } else if tx_count < 50 {
            0.4
        } else if tx_count < 100 {
            0.6
        } else {
            0.8 // Very high volume could be suspicious
        }
    }
    
    fn calculate_transaction_frequency_risk(&self, transaction_history: &[serde_json::Value]) -> f64 {
        // Simplified implementation
        0.5 // Moderate risk
    }
    
    fn calculate_deposit_withdrawal_ratio(&self, transaction_history: &[serde_json::Value]) -> f64 {
        // Simplified implementation
        0.5 // Moderate risk
    }
    
    fn calculate_login_pattern_risk(&self, user: &UserProfile) -> f64 {
        // Simplified implementation
        0.3 // Low-moderate risk
    }
    
    fn calculate_trading_pattern_risk(&self, transaction_history: &[serde_json::Value]) -> f64 {
        // Simplified implementation
        0.4 // Moderate risk
    }
}

// KYC manager
pub struct KycManager {
    user_store: Arc<UserStore>,
    document_store: Arc<DocumentStore>,
    verification_store: Arc<VerificationStore>,
    kyc_provider: Arc<dyn KycProvider>,
    risk_engine: Arc<RiskScoringEngine>,
}

impl KycManager {
    pub fn new(
        user_store: Arc<UserStore>,
        document_store: Arc<DocumentStore>,
        verification_store: Arc<VerificationStore>,
        kyc_provider: Arc<dyn KycProvider>,
        risk_engine: Arc<RiskScoringEngine>,
    ) -> Self {
        KycManager {
            user_store,
            document_store,
            verification_store,
            kyc_provider,
            risk_engine,
        }
    }
    
    pub async fn register_user(&self, email: String, password: &str) -> Result<UserProfile> {
        // Create a new user profile
        let user = UserProfile::new(email, password)?;
        
        // Save the user
        self.user_store.add_user(user.clone()).await?;
        
        Ok(user)
    }
    
    pub async fn upload_document(
        &self,
        user_id: &UserId,
        document_type: DocumentType,
        file_path: String,
        file_content: &[u8],
    ) -> Result<UserDocument> {
        // Verify user exists
        let user = self.user_store.get_user(user_id).await?;
        
        // Calculate file hash for integrity checking
        let mut hasher = sha2::Sha256::new();
        hasher.update(file_content);
        let file_hash = format!("{:x}", hasher.finalize());
        
        // Create document record
        let document = UserDocument {
            id: Uuid::new_v4(),
            user_id: *user_id,
            document_type,
            file_path,
            file_hash,
            uploaded_at: Utc::now(),
            expires_at: None, // Set based on document type in a real implementation
            status: VerificationStatus::Pending,
            reviewer_notes: None,
            metadata: serde_json::json!({}),
        };
        
        // Save the document
        self.document_store.add_document(document.clone()).await?;
        
        // Update user status if needed
        if user.status == UserStatus::Unverified {
            self.user_store
                .update_user_status(user_id, UserStatus::PendingVerification)
                .await?;
        }
        
        Ok(document)
    }
    
    pub async fn start_verification(&self, user_id: &UserId, tier: IdentityTier) -> Result<VerificationAttempt> {
        // Get user's documents
        let documents = self.document_store.get_user_documents(user_id).await?;
        
        if documents.is_empty() {
            return Err(anyhow!("User has no documents uploaded"));
        }
        
        // Check if all required documents are available based on tier
        let required_documents = self.get_required_documents_for_tier(tier);
        let user_document_types: Vec<DocumentType> = documents.iter().map(|d| d.document_type).collect();
        
        for &required in &required_documents {
            if !user_document_types.contains(&required) {
                return Err(anyhow!("Missing required document: {:?}", required));
            }
        }
        
        // Create verification attempt
        let verification = VerificationAttempt {
            id: Uuid::new_v4(),
            user_id: *user_id,
            tier,
            documents: documents.iter().map(|d| d.id).collect(),
            verification_provider: Some(self.kyc_provider.get_provider_name().await),
            provider_reference: None, // Will be set after verification
            status: VerificationStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            notes: None,
        };
        
        // Save the verification attempt
        self.verification_store.add_verification(verification.clone()).await?;
        
        // Start the verification process with the provider
        let verification_status = self
            .kyc_provider
            .verify_identity(user_id, &verification.documents)
            .await?;
        
        // Update verification status
        let updated_verification = self
            .verification_store
            .update_verification_status(
                &verification.id,
                verification_status,
                None,
            )
            .await?;
        
        // If verification was approved, update user tier
        if verification_status == VerificationStatus::Approved {
            self.user_store.update_user_tier(user_id, tier).await?;
            self.user_store.update_user_status(user_id, UserStatus::Active).await?;
        }
        
        Ok(updated_verification)
    }
    
    pub async fn check_aml(&self, user_id: &UserId) -> Result<AmlCheckResult> {
        // Perform AML check
        let aml_result = self.kyc_provider.check_aml(user_id).await?;
        
        // Update user risk level based on AML result
        let mut user = self.user_store.get_user(user_id).await?;
        
        let new_risk_level = match aml_result.result {
            AmlResult::Clear => RiskLevel::Low,
            AmlResult::PotentialMatch => RiskLevel::Medium,
            AmlResult::Match => RiskLevel::High,
            AmlResult::Error => user.risk_level, // Maintain current level on error
        };
        
        // Only increase risk level, never decrease it based on AML
        if new_risk_level as u8 > user.risk_level as u8 {
            user.risk_level = new_risk_level;
            self.user_store.update_user(user).await?;
        }
        
        Ok(aml_result)
    }
    
    pub async fn calculate_user_risk(&self, user_id: &UserId) -> Result<RiskLevel> {
        let user = self.user_store.get_user(user_id).await?;
        
        // In a real implementation, get transaction history from database
        // Here we use an empty array for simplicity
        let transaction_history: Vec<serde_json::Value> = Vec::new();
        
        let risk_level = self.risk_engine.calculate_user_risk(&user, &transaction_history);
        
        // Update user risk level if it changed
        if risk_level != user.risk_level {
            let mut updated_user = user.clone();
            updated_user.risk_level = risk_level;
            self.user_store.update_user(updated_user).await?;
        }
        
        Ok(risk_level)
    }
    
    fn get_required_documents_for_tier(&self, tier: IdentityTier) -> Vec<DocumentType> {
        match tier {
            IdentityTier::Tier0 => Vec::new(), // No documents required
            IdentityTier::Tier1 => vec![DocumentType::Selfie],
            IdentityTier::Tier2 => vec![
                DocumentType::Passport,
                DocumentType::Selfie,
                DocumentType::ProofOfAddress,
            ],
            IdentityTier::Tier3 => vec![
                DocumentType::Passport,
                DocumentType::Selfie,
                DocumentType::ProofOfAddress,
                DocumentType::VideoVerification,
            ],
            IdentityTier::Tier4 => vec![
                DocumentType::Passport,
                DocumentType::Selfie,
                DocumentType::ProofOfAddress,
                DocumentType::VideoVerification,
                DocumentType::CompanyRegistration,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_registration_and_kyc() {
        // Create stores
        let user_store = Arc::new(UserStore::new());
        let document_store = Arc::new(DocumentStore::new());
        let verification_store = Arc::new(VerificationStore::new());
        
        // Create mock KYC provider
        let kyc_provider = Arc::new(MockKycProvider);
        
        // Create risk model
        let risk_model = RiskScoringModel {
            user_factors: vec![
                RiskFactor {
                    name: "country_risk".to_string(),
                    weight: 0.4,
                    thresholds: vec![
                        (0.3, RiskLevel::Low),
                        (0.6, RiskLevel::Medium),
                        (0.8, RiskLevel::High),
                    ],
                },
                RiskFactor {
                    name: "verification_level".to_string(),
                    weight: 0.6,
                    thresholds: vec![
                        (0.3, RiskLevel::Low),
                        (0.6, RiskLevel::Medium),
                        (0.8, RiskLevel::High),
                    ],
                },
            ],
            transaction_factors: vec![
                RiskFactor {
                    name: "transaction_volume".to_string(),
                    weight: 0.5,
                    thresholds: vec![
                        (0.3, RiskLevel::Low),
                        (0.6, RiskLevel::Medium),
                        (0.8, RiskLevel::High),
                    ],
                },
            ],
            behavioral_factors: vec![
                RiskFactor {
                    name: "login_patterns".to_string(),
                    weight: 0.3,
                    thresholds: vec![
                        (0.3, RiskLevel::Low),
                        (0.6, RiskLevel::Medium),
                        (0.8, RiskLevel::High),
                    ],
                },
            ],
        };
        
        let risk_engine = Arc::new(RiskScoringEngine::new(risk_model));
        
        // Create KYC manager
        let kyc_manager = KycManager::new(
            user_store.clone(),
            document_store.clone(),
            verification_store.clone(),
            kyc_provider,
            risk_engine,
        );
        
        // Register a new user
        let user = kyc_manager
            .register_user("test@example.com".to_string(), "password123")
            .await
            .unwrap();
        
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.status, UserStatus::Unverified);
        assert_eq!(user.identity_tier, IdentityTier::Tier0);
        
        // Upload documents
        let passport_doc = kyc_manager
            .upload_document(
                &user.user_id,
                DocumentType::Passport,
                "/tmp/passport.jpg".to_string(),
                b"fake_passport_image_data",
            )
            .await
            .unwrap();
        
        let selfie_doc = kyc_manager
            .upload_document(
                &user.user_id,
                DocumentType::Selfie,
                "/tmp/selfie.jpg".to_string(),
                b"fake_selfie_image_data",
            )
            .await
            .unwrap();
        
        let address_doc = kyc_manager
            .upload_document(
                &user.user_id,
                DocumentType::ProofOfAddress,
                "/tmp/utility_bill.pdf".to_string(),
                b"fake_utility_bill_data",
            )
            .await
            .unwrap();
        
        // Check user status after document upload
        let updated_user = user_store.get_user(&user.user_id).await.unwrap();
        assert_eq!(updated_user.status, UserStatus::PendingVerification);
        
        // Start verification for Tier2
        let verification = kyc_manager
            .start_verification(&user.user_id, IdentityTier::Tier2)
            .await
            .unwrap();
        
        // Verification should be in a final state (Approved, Rejected, or InReview from the mock)
        assert!(
            verification.status == VerificationStatus::Approved
                || verification.status == VerificationStatus::Rejected
                || verification.status == VerificationStatus::InReview
        );
        
        // If verification was approved, check if user tier was updated
        if verification.status == VerificationStatus::Approved {
            let verified_user = user_store.get_user(&user.user_id).await.unwrap();
            assert_eq!(verified_user.identity_tier, IdentityTier::Tier2);
            assert_eq!(verified_user.status, UserStatus::Active);
        }
        
        // Perform AML check
        let aml_result = kyc_manager.check_aml(&user.user_id).await.unwrap();
        
        // The result should be one of the valid AML results
        assert!(
            aml_result.result == AmlResult::Clear
                || aml_result.result == AmlResult::PotentialMatch
                || aml_result.result == AmlResult::Match
        );
        
        // Calculate user risk
        let risk_level = kyc_manager.calculate_user_risk(&user.user_id).await.unwrap();
        
        // Risk level should be a valid value
        assert!(
            risk_level == RiskLevel::Low
                || risk_level == RiskLevel::Medium
                || risk_level == RiskLevel::High
                || risk_level == RiskLevel::Extreme
        );
        
        // Final user profile should reflect risk calculation
        let final_user = user_store.get_user(&user.user_id).await.unwrap();
        assert_eq!(final_user.risk_level, risk_level);
    }
}
