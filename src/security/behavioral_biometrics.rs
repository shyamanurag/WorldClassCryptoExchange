// WorldClass Crypto Exchange: Behavioral Biometrics Implementation
// This file contains the behavioral biometrics security system

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::sync::Mutex;

///////////////////////////////////////////////////////////////////////////////
// Behavioral Biometrics Implementation
///////////////////////////////////////////////////////////////////////////////

// User behavior profile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserBehaviorProfile {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub device_fingerprints: Vec<DeviceFingerprint>,
    pub typing_patterns: Option<TypingPatterns>,
    pub mouse_patterns: Option<MousePatterns>,
    pub session_patterns: SessionPatterns,
    pub transaction_patterns: TransactionPatterns,
    pub confidence_score: f64, // 0.0 to 1.0
    pub anomaly_score: f64,    // 0.0 to 1.0
    pub status: ProfileStatus,
}

// Profile status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileStatus {
    Learning,
    Active,
    Suspended,
    Locked,
}

// Device fingerprint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub id: String,
    pub name: Option<String>,
    pub user_agent: String,
    pub operating_system: String,
    pub browser: String,
    pub browser_version: String,
    pub screen_resolution: String,
    pub color_depth: u32,
    pub timezone: String,
    pub language: String,
    pub plugins: Vec<String>,
    pub canvas_fingerprint: String,
    pub webgl_fingerprint: String,
    pub fonts: Vec<String>,
    pub ip_addresses: Vec<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_trusted: bool,
    pub risk_score: f64, // 0.0 to 1.0
}

// Typing patterns
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypingPatterns {
    pub average_typing_speed: f64, // Characters per minute
    pub typing_rhythm: Vec<f64>,   // Inter-key timing patterns
    pub key_down_duration: HashMap<String, f64>, // Average time each key is held down
    pub error_rate: f64,           // Typos per character
    pub backspace_usage: f64,      // Backspace usage per character
    pub common_bigrams: HashMap<String, f64>, // Common two-letter combinations and their speeds
    pub keypress_force: HashMap<String, f64>, // For devices with pressure sensitivity
    pub last_updated: DateTime<Utc>,
}

// Mouse patterns
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MousePatterns {
    pub average_speed: f64,        // Pixels per second
    pub acceleration_profile: Vec<f64>, // Acceleration patterns
    pub click_patterns: HashMap<String, f64>, // Speed of clicks in different UI areas
    pub cursor_path_straightness: f64, // How straight the cursor moves (0.0 to 1.0)
    pub hover_behavior: HashMap<String, f64>, // Hover duration in different UI areas
    pub scroll_behavior: ScrollBehavior,
    pub last_updated: DateTime<Utc>,
}

// Scroll behavior
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScrollBehavior {
    pub average_scroll_speed: f64,  // Pixels per second
    pub scroll_direction_ratio: f64, // Ratio of down-scrolls to up-scrolls
    pub scroll_distance: f64,        // Average scroll distance per action
    pub scroll_pause_duration: f64,  // Average time between scrolls in ms
}

// Session patterns
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionPatterns {
    pub typical_session_duration: f64, // In minutes
    pub session_time_distribution: HashMap<String, f64>, // Hour -> frequency
    pub typical_days: HashMap<String, f64>, // Day of week -> frequency
    pub typical_locations: Vec<GeoLocation>,
    pub typical_ip_ranges: Vec<String>,
    pub device_usage_ratios: HashMap<String, f64>, // Device ID -> usage ratio
    pub average_actions_per_session: f64,
    pub typical_session_flow: Vec<String>, // Common navigation patterns
    pub last_updated: DateTime<Utc>,
}

// Geo location
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: String,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub frequency: f64, // 0.0 to 1.0
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

// Transaction patterns
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPatterns {
    pub typical_transaction_amounts: Vec<AmountRange>,
    pub typical_transaction_times: HashMap<String, f64>, // Hour -> frequency
    pub typical_assets: HashMap<String, f64>, // Asset -> frequency
    pub typical_trading_pairs: HashMap<String, f64>, // Trading pair -> frequency
    pub typical_transaction_frequency: f64, // Transactions per day
    pub typical_withdrawal_frequency: f64, // Withdrawals per week
    pub typical_withdrawal_addresses: HashMap<String, f64>, // Address -> frequency
    pub typical_deposit_sources: HashMap<String, f64>, // Source -> frequency
    pub risk_appetite: f64, // 0.0 (conservative) to 1.0 (aggressive)
    pub last_updated: DateTime<Utc>,
}

// Amount range
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmountRange {
    pub min: f64,
    pub max: f64,
    pub frequency: f64, // 0.0 to 1.0
}

// Behavior observation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorObservation {
    pub id: String,
    pub user_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub observation_type: ObservationType,
    pub device_fingerprint: Option<DeviceFingerprint>,
    pub typing_data: Option<TypingData>,
    pub mouse_data: Option<MouseData>,
    pub session_data: Option<SessionData>,
    pub transaction_data: Option<TransactionData>,
    pub anomaly_score: Option<f64>,
    pub is_anomaly: Option<bool>,
}

// Observation type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationType {
    DeviceFingerprint,
    TypingBehavior,
    MouseBehavior,
    SessionBehavior,
    TransactionBehavior,
}

// Typing data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypingData {
    pub keystroke_times: Vec<KeystrokeTime>,
    pub context: String, // What field was being typed in
    pub typing_duration: u64, // In milliseconds
    pub error_count: u32,
    pub backspace_count: u32,
}

// Keystroke time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeystrokeTime {
    pub key: String,
    pub key_down_time: u64, // Unix timestamp in milliseconds
    pub key_up_time: u64,   // Unix timestamp in milliseconds
    pub key_code: u32,
}

// Mouse data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseData {
    pub mouse_movements: Vec<MouseMovement>,
    pub mouse_clicks: Vec<MouseClick>,
    pub scrolls: Vec<ScrollEvent>,
    pub total_distance: f64,
    pub total_duration: u64, // In milliseconds
}

// Mouse movement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseMovement {
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub x: u32,
    pub y: u32,
    pub velocity: Option<f64>,
    pub acceleration: Option<f64>,
}

// Mouse click
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseClick {
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub x: u32,
    pub y: u32,
    pub button: String, // "left", "right", "middle"
    pub element_type: String, // What was clicked
    pub context: String, // Context of the click
}

// Scroll event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScrollEvent {
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub delta_x: i32,
    pub delta_y: i32,
    pub speed: f64,
}

// Session data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub ip_address: String,
    pub location: Option<GeoLocation>,
    pub device_id: String,
    pub user_agent: String,
    pub session_actions: Vec<SessionAction>,
    pub referrer: Option<String>,
    pub entry_page: String,
}

// Session action
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionAction {
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub page: String,
    pub context: String,
    pub duration: Option<u64>, // In milliseconds
}

// Transaction data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub timestamp: DateTime<Utc>,
    pub transaction_type: TransactionType,
    pub asset: String,
    pub amount: f64,
    pub destination: Option<String>,
    pub source: Option<String>,
    pub fee: Option<f64>,
    pub status: String,
    pub ip_address: String,
    pub device_id: String,
}

// Transaction type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Trade,
    InternalTransfer,
    FiatConversion,
}

// Authentication thresholds
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticationThresholds {
    pub typing_match_threshold: f64,         // 0.0 to 1.0
    pub mouse_match_threshold: f64,          // 0.0 to 1.0
    pub session_match_threshold: f64,        // 0.0 to 1.0
    pub transaction_match_threshold: f64,    // 0.0 to 1.0
    pub overall_match_threshold: f64,        // 0.0 to 1.0
    pub high_risk_threshold: f64,            // 0.0 to 1.0
    pub anomaly_threshold: f64,              // 0.0 to 1.0
    pub learning_phase_observations: u32,    // Number of observations needed for learning phase
    pub continuous_learning_rate: f64,       // 0.0 to 1.0
    pub feature_weights: FeatureWeights,
}

// Feature weights for scoring
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureWeights {
    pub typing_weight: f64,
    pub mouse_weight: f64,
    pub session_weight: f64,
    pub transaction_weight: f64,
    pub device_weight: f64,
    pub location_weight: f64,
    pub time_pattern_weight: f64,
    pub amount_pattern_weight: f64,
}

// Behavioral authentication result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub user_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub authenticated: bool,
    pub confidence_score: f64,
    pub risk_score: f64,
    pub anomaly_detected: bool,
    pub anomalous_features: Vec<String>,
    pub auth_factors_used: Vec<String>,
    pub recommendation: AuthRecommendation,
}

// Authentication recommendation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthRecommendation {
    Allow,
    AdditionalFactorRequired,
    ManualReview,
    Block,
}

// Behavioral biometrics service
pub struct BehavioralBiometricsService {
    profiles: Arc<RwLock<HashMap<String, UserBehaviorProfile>>>,
    observations: Arc<RwLock<VecDeque<BehaviorObservation>>>,
    thresholds: Arc<RwLock<AuthenticationThresholds>>,
    profile_by_user: Arc<RwLock<HashMap<String, String>>>, // User ID -> Profile ID
    max_observations: usize,
    security_monitoring: Arc<dyn SecurityMonitoringInterface>,
}

// Security monitoring interface for dependency injection
#[async_trait::async_trait]
pub trait SecurityMonitoringInterface: Send + Sync {
    async fn log_event(
        &self,
        event_type: &str,
        user_id: Option<&str>,
        ip_address: Option<&str>,
        resource_id: Option<&str>,
        severity: &str,
        details: serde_json::Value,
    ) -> String;
}

impl BehavioralBiometricsService {
    pub fn new(
        security_monitoring: Arc<dyn SecurityMonitoringInterface>,
        max_observations: usize,
    ) -> Self {
        // Default authentication thresholds
        let default_thresholds = AuthenticationThresholds {
            typing_match_threshold: 0.7,
            mouse_match_threshold: 0.65,
            session_match_threshold: 0.75,
            transaction_match_threshold: 0.8,
            overall_match_threshold: 0.75,
            high_risk_threshold: 0.9,
            anomaly_threshold: 0.8,
            learning_phase_observations: 20,
            continuous_learning_rate: 0.1,
            feature_weights: FeatureWeights {
                typing_weight: 0.25,
                mouse_weight: 0.15,
                session_weight: 0.2,
                transaction_weight: 0.3,
                device_weight: 0.3,
                location_weight: 0.25,
                time_pattern_weight: 0.1,
                amount_pattern_weight: 0.2,
            },
        };
        
        BehavioralBiometricsService {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            observations: Arc::new(RwLock::new(VecDeque::with_capacity(max_observations))),
            thresholds: Arc::new(RwLock::new(default_thresholds)),
            profile_by_user: Arc::new(RwLock::new(HashMap::new())),
            max_observations,
            security_monitoring,
        }
    }
    
    // Create a new user behavior profile
    pub async fn create_profile(&self, user_id: &str) -> Result<String, String> {
        // Check if user already has a profile
        {
            let profile_by_user = self.profile_by_user.read().unwrap();
            if profile_by_user.contains_key(user_id) {
                return Err("User already has a behavior profile".to_string());
            }
        }
        
        // Create new profile
        let profile_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let profile = UserBehaviorProfile {
            id: profile_id.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            updated_at: now,
            device_fingerprints: Vec::new(),
            typing_patterns: None,
            mouse_patterns: None,
            session_patterns: SessionPatterns {
                typical_session_duration: 0.0,
                session_time_distribution: HashMap::new(),
                typical_days: HashMap::new(),
                typical_locations: Vec::new(),
                typical_ip_ranges: Vec::new(),
                device_usage_ratios: HashMap::new(),
                average_actions_per_session: 0.0,
                typical_session_flow: Vec::new(),
                last_updated: now,
            },
            transaction_patterns: TransactionPatterns {
                typical_transaction_amounts: Vec::new(),
                typical_transaction_times: HashMap::new(),
                typical_assets: HashMap::new(),
                typical_trading_pairs: HashMap::new(),
                typical_transaction_frequency: 0.0,
                typical_withdrawal_frequency: 0.0,
                typical_withdrawal_addresses: HashMap::new(),
                typical_deposit_sources: HashMap::new(),
                risk_appetite: 0.5,
                last_updated: now,
            },
            confidence_score: 0.0,
            anomaly_score: 0.0,
            status: ProfileStatus::Learning,
        };
        
        // Store profile
        {
            let mut profiles = self.profiles.write().unwrap();
            profiles.insert(profile_id.clone(), profile);
        }
        
        // Map user to profile
        {
            let mut profile_by_user = self.profile_by_user.write().unwrap();
            profile_by_user.insert(user_id.to_string(), profile_id.clone());
        }
        
        Ok(profile_id)
    }
    
    // Record a behavior observation
    pub async fn record_observation(
        &self,
        observation: BehaviorObservation,
    ) -> Result<(), String> {
        // Store observation
        {
            let mut observations = self.observations.write().unwrap();
            observations.push_back(observation.clone());
            
            // Limit number of stored observations
            while observations.len() > self.max_observations {
                observations.pop_front();
            }
        }
        
        // Update user profile
        let profile_id = {
            let profile_by_user = self.profile_by_user.read().unwrap();
            profile_by_user.get(&observation.user_id)
                .cloned()
                .ok_or_else(|| "User profile not found".to_string())?
        };
        
        let mut should_update_profile = false;
        let mut anomaly_detected = false;
        let mut anomaly_score = 0.0;
        
        // Process observation based on type
        match observation.observation_type {
            ObservationType::DeviceFingerprint => {
                if let Some(device_data) = &observation.device_fingerprint {
                    should_update_profile = true;
                    
                    // Check if this is a new device or an existing one
                    let is_new_device = {
                        let profiles = self.profiles.read().unwrap();
                        let profile = profiles.get(&profile_id)
                            .ok_or_else(|| "Profile not found".to_string())?;
                        
                        // Check if device fingerprint is already known
                        !profile.device_fingerprints.iter()
                            .any(|d| d.canvas_fingerprint == device_data.canvas_fingerprint)
                    };
                    
                    // If it's a new device, flag it for review
                    if is_new_device {
                        anomaly_score = 0.7; // New device is suspicious but not immediately alarming
                        
                        // Log security event for new device
                        self.security_monitoring.log_event(
                            "NewDeviceDetected",
                            Some(&observation.user_id),
                            None,
                            Some(&profile_id),
                            "Medium",
                            serde_json::json!({
                                "device_fingerprint": device_data,
                                "session_id": observation.session_id,
                            }),
                        ).await;
                    }
                }
            },
            ObservationType::TypingBehavior => {
                if let Some(typing_data) = &observation.typing_data {
                    should_update_profile = true;
                    
                    // Calculate anomaly score for typing pattern
                    anomaly_score = self.calculate_typing_anomaly_score(&profile_id, typing_data).await?;
                    
                    // If typing pattern is highly anomalous, flag it
                    if anomaly_score > self.thresholds.read().unwrap().anomaly_threshold {
                        anomaly_detected = true;
                        
                        // Log security event for anomalous typing
                        self.security_monitoring.log_event(
                            "AnomalousTypingDetected",
                            Some(&observation.user_id),
                            None,
                            Some(&profile_id),
                            "Medium",
                            serde_json::json!({
                                "anomaly_score": anomaly_score,
                                "session_id": observation.session_id,
                            }),
                        ).await;
                    }
                }
            },
            ObservationType::MouseBehavior => {
                if let Some(mouse_data) = &observation.mouse_data {
                    should_update_profile = true;
                    
                    // Calculate anomaly score for mouse pattern
                    anomaly_score = self.calculate_mouse_anomaly_score(&profile_id, mouse_data).await?;
                    
                    // If mouse pattern is highly anomalous, flag it
                    if anomaly_score > self.thresholds.read().unwrap().anomaly_threshold {
                        anomaly_detected = true;
                        
                        // Log security event for anomalous mouse behavior
                        self.security_monitoring.log_event(
                            "AnomalousMouseBehaviorDetected",
                            Some(&observation.user_id),
                            None,
                            Some(&profile_id),
                            "Medium",
                            serde_json::json!({
                                "anomaly_score": anomaly_score,
                                "session_id": observation.session_id,
                            }),
                        ).await;
                    }
                }
            },
            ObservationType::SessionBehavior => {
                if let Some(session_data) = &observation.session_data {
                    should_update_profile = true;
                    
                    // Calculate anomaly score for session pattern
                    anomaly_score = self.calculate_session_anomaly_score(&profile_id, session_data).await?;
                    
                    // If session pattern is highly anomalous, flag it
                    if anomaly_score > self.thresholds.read().unwrap().anomaly_threshold {
                        anomaly_detected = true;
                        
                        // Log security event for anomalous session
                        self.security_monitoring.log_event(
                            "AnomalousSessionDetected",
                            Some(&observation.user_id),
                            None,
                            Some(&profile_id),
                            "High",
                            serde_json::json!({
                                "anomaly_score": anomaly_score,
                                "session_id": observation.session_id,
                                "ip_address": session_data.ip_address,
                            }),
                        ).await;
                    }
                }
            },
            ObservationType::TransactionBehavior => {
                if let Some(transaction_data) = &observation.transaction_data {
                    should_update_profile = true;
                    
                    // Calculate anomaly score for transaction pattern
                    anomaly_score = self.calculate_transaction_anomaly_score(&profile_id, transaction_data).await?;
                    
                    // If transaction pattern is highly anomalous, flag it
                    if anomaly_score > self.thresholds.read().unwrap().anomaly_threshold {
                        anomaly_detected = true;
                        
                        // Log security event for anomalous transaction
                        self.security_monitoring.log_event(
                            "AnomalousTransactionDetected",
                            Some(&observation.user_id),
                            Some(&transaction_data.ip_address),
                            Some(&profile_id),
                            "Critical",
                            serde_json::json!({
                                "anomaly_score": anomaly_score,
                                "transaction_type": format!("{:?}", transaction_data.transaction_type),
                                "amount": transaction_data.amount,
                                "asset": transaction_data.asset,
                            }),
                        ).await;
                    }
                }
            },
        }
        
        // Update profile if needed
        if should_update_profile {
            self.update_profile(&profile_id, &observation).await?;
        }
        
        // Update observation with anomaly score
        {
            let mut observations = self.observations.write().unwrap();
            if let Some(obs) = observations.iter_mut().rev().find(|o| o.id == observation.id) {
                obs.anomaly_score = Some(anomaly_score);
                obs.is_anomaly = Some(anomaly_detected);
            }
        }
        
        Ok(())
    }
    
    // Update user profile based on observation
    async fn update_profile(
        &self,
        profile_id: &str,
        observation: &BehaviorObservation,
    ) -> Result<(), String> {
        let mut profiles = self.profiles.write().unwrap();
        
        let profile = profiles.get_mut(profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        // Update profile based on observation type
        match observation.observation_type {
            ObservationType::DeviceFingerprint => {
                if let Some(device_data) = &observation.device_fingerprint {
                    // Check if device already exists
                    let device_index = profile.device_fingerprints.iter()
                        .position(|d| d.canvas_fingerprint == device_data.canvas_fingerprint);
                    
                    if let Some(index) = device_index {
                        // Update existing device
                        let device = &mut profile.device_fingerprints[index];
                        device.last_seen = Utc::now();
                        device.user_agent = device_data.user_agent.clone();
                        device.browser = device_data.browser.clone();
                        device.browser_version = device_data.browser_version.clone();
                        device.ip_addresses.extend(
                            device_data.ip_addresses.iter()
                                .filter(|ip| !device.ip_addresses.contains(ip))
                                .cloned()
                        );
                    } else {
                        // Add new device
                        profile.device_fingerprints.push(device_data.clone());
                    }
                    
                    // Update device usage ratios
                    let total_devices = profile.device_fingerprints.len() as f64;
                    profile.session_patterns.device_usage_ratios.clear();
                    
                    for device in &profile.device_fingerprints {
                        profile.session_patterns.device_usage_ratios.insert(
                            device.id.clone(),
                            1.0 / total_devices, // Simple equal distribution for now
                        );
                    }
                }
            },
            ObservationType::TypingBehavior => {
                if let Some(typing_data) = &observation.typing_data {
                    // Update typing patterns
                    let typing_patterns = if let Some(patterns) = &mut profile.typing_patterns {
                        patterns
                    } else {
                        profile.typing_patterns = Some(TypingPatterns {
                            average_typing_speed: 0.0,
                            typing_rhythm: Vec::new(),
                            key_down_duration: HashMap::new(),
                            error_rate: 0.0,
                            backspace_usage: 0.0,
                            common_bigrams: HashMap::new(),
                            keypress_force: HashMap::new(),
                            last_updated: Utc::now(),
                        });
                        profile.typing_patterns.as_mut().unwrap()
                    };
                    
                    // Calculate typing speed (characters per minute)
                    let char_count = typing_data.keystroke_times.len() as f64;
                    let duration_minutes = typing_data.typing_duration as f64 / 60000.0;
                    let typing_speed = if duration_minutes > 0.0 {
                        char_count / duration_minutes
                    } else {
                        0.0
                    };
                    
                    // Update average typing speed (weighted average)
                    typing_patterns.average_typing_speed = 
                        (typing_patterns.average_typing_speed * 0.7) + (typing_speed * 0.3);
                    
                    // Update key down duration
                    for keystroke in &typing_data.keystroke_times {
                        let key_down_time = (keystroke.key_up_time - keystroke.key_down_time) as f64;
                        
                        let entry = typing_patterns.key_down_duration
                            .entry(keystroke.key.clone())
                            .or_insert(key_down_time);
                        
                        // Weighted average
                        *entry = (*entry * 0.7) + (key_down_time * 0.3);
                    }
                    
                    // Update error rate
                    let error_rate = if char_count > 0.0 {
                        typing_data.error_count as f64 / char_count
                    } else {
                        0.0
                    };
                    
                    typing_patterns.error_rate = 
                        (typing_patterns.error_rate * 0.7) + (error_rate * 0.3);
                    
                    // Update backspace usage
                    let backspace_rate = if char_count > 0.0 {
                        typing_data.backspace_count as f64 / char_count
                    } else {
                        0.0
                    };
                    
                    typing_patterns.backspace_usage = 
                        (typing_patterns.backspace_usage * 0.7) + (backspace_rate * 0.3);
                    
                    // Update typing rhythm
                    if typing_data.keystroke_times.len() >= 2 {
                        let mut inter_key_times = Vec::new();
                        
                        for i in 1..typing_data.keystroke_times.len() {
                            let prev = &typing_data.keystroke_times[i-1];
                            let curr = &typing_data.keystroke_times[i];
                            
                            let time_diff = curr.key_down_time as i64 - prev.key_up_time as i64;
                            if time_diff > 0 && time_diff < 1000 { // Filter out pauses
                                inter_key_times.push(time_diff as f64);
                            }
                        }
                        
                        // Replace or merge typing rhythm
                        if typing_patterns.typing_rhythm.is_empty() {
                            typing_patterns.typing_rhythm = inter_key_times;
                        } else {
                            // Keep only the most recent 100 rhythm samples
                            let mut combined = typing_patterns.typing_rhythm.clone();
                            combined.extend(inter_key_times);
                            if combined.len() > 100 {
                                combined.drain(0..combined.len() - 100);
                            }
                            typing_patterns.typing_rhythm = combined;
                        }
                    }
                    
                    // Update common bigrams
                    if typing_data.keystroke_times.len() >= 2 {
                        for i in 1..typing_data.keystroke_times.len() {
                            let prev = &typing_data.keystroke_times[i-1];
                            let curr = &typing_data.keystroke_times[i];
                            
                            let bigram = format!("{}{}", prev.key, curr.key);
                            let time_diff = curr.key_down_time as i64 - prev.key_down_time as i64;
                            
                            if time_diff > 0 && time_diff < 1000 { // Filter out pauses
                                let entry = typing_patterns.common_bigrams
                                    .entry(bigram)
                                    .or_insert(time_diff as f64);
                                
                                // Weighted average
                                *entry = (*entry * 0.7) + ((time_diff as f64) * 0.3);
                            }
                        }
                    }
                    
                    typing_patterns.last_updated = Utc::now();
                }
            },
            ObservationType::MouseBehavior => {
                if let Some(mouse_data) = &observation.mouse_data {
                    // Update mouse patterns
                    let mouse_patterns = if let Some(patterns) = &mut profile.mouse_patterns {
                        patterns
                    } else {
                        profile.mouse_patterns = Some(MousePatterns {
                            average_speed: 0.0,
                            acceleration_profile: Vec::new(),
                            click_patterns: HashMap::new(),
                            cursor_path_straightness: 0.0,
                            hover_behavior: HashMap::new(),
                            scroll_behavior: ScrollBehavior {
                                average_scroll_speed: 0.0,
                                scroll_direction_ratio: 0.5,
                                scroll_distance: 0.0,
                                scroll_pause_duration: 0.0,
                            },
                            last_updated: Utc::now(),
                        });
                        profile.mouse_patterns.as_mut().unwrap()
                    };
                    
                    // Calculate mouse speed
                    if mouse_data.total_duration > 0 {
                        let speed = mouse_data.total_distance / (mouse_data.total_duration as f64 / 1000.0);
                        mouse_patterns.average_speed = 
                            (mouse_patterns.average_speed * 0.7) + (speed * 0.3);
                    }
                    
                    // Update click patterns
                    for click in &mouse_data.mouse_clicks {
                        let context_key = format!("{}:{}", click.element_type, click.context);
                        
                        let entry = mouse_patterns.click_patterns
                            .entry(context_key)
                            .or_insert(1.0);
                        
                        *entry += 1.0;
                    }
                    
                    // Normalize click patterns
                    let total_clicks = mouse_patterns.click_patterns.values().sum::<f64>();
                    if total_clicks > 0.0 {
                        for value in mouse_patterns.click_patterns.values_mut() {
                            *value /= total_clicks;
                        }
                    }
                    
                    // Update cursor path straightness
                    if mouse_data.mouse_movements.len() >= 2 {
                        let mut total_actual_distance = 0.0;
                        let mut total_direct_distance = 0.0;
                        
                        for i in 1..mouse_data.mouse_movements.len() {
                            let prev = &mouse_data.mouse_movements[i-1];
                            let curr = &mouse_data.mouse_movements[i];
                            
                            let actual_distance = (
                                ((curr.x as f64 - prev.x as f64).powi(2) + 
                                 (curr.y as f64 - prev.y as f64).powi(2)).sqrt()
                            );
                            
                            total_actual_distance += actual_distance;
                        }
                        
                        let first = &mouse_data.mouse_movements.first().unwrap();
                        let last = &mouse_data.mouse_movements.last().unwrap();
                        
                        total_direct_distance = (
                            ((last.x as f64 - first.x as f64).powi(2) + 
                             (last.y as f64 - first.y as f64).powi(2)).sqrt()
                        );
                        
                        if total_actual_distance > 0.0 {
                            let straightness = total_direct_distance / total_actual_distance;
                            // Update with weighted average
                            mouse_patterns.cursor_path_straightness = 
                                (mouse_patterns.cursor_path_straightness * 0.7) + (straightness * 0.3);
                        }
                    }
                    
                    // Update scroll behavior
                    if !mouse_data.scrolls.is_empty() {
                        let scroll_behavior = &mut mouse_patterns.scroll_behavior;
                        
                        // Calculate scroll metrics
                        let total_scrolls = mouse_data.scrolls.len() as f64;
                        let total_scroll_distance: f64 = mouse_data.scrolls.iter()
                            .map(|s| (s.delta_y.abs() + s.delta_x.abs()) as f64)
                            .sum();
                        
                        let avg_scroll_distance = total_scroll_distance / total_scrolls;
                        
                        // Update scroll distance with weighted average
                        scroll_behavior.scroll_distance = 
                            (scroll_behavior.scroll_distance * 0.7) + (avg_scroll_distance * 0.3);
                        
                        // Calculate scroll speed
                        let total_scroll_speed: f64 = mouse_data.scrolls.iter()
                            .map(|s| s.speed)
                            .sum();
                        
                        let avg_scroll_speed = total_scroll_speed / total_scrolls;
                        
                        // Update scroll speed with weighted average
                        scroll_behavior.average_scroll_speed = 
                            (scroll_behavior.average_scroll_speed * 0.7) + (avg_scroll_speed * 0.3);
                        
                        // Calculate scroll direction ratio
                        let down_scrolls = mouse_data.scrolls.iter()
                            .filter(|s| s.delta_y > 0)
                            .count() as f64;
                        
                        let direction_ratio = down_scrolls / total_scrolls;
                        
                        // Update direction ratio with weighted average
                        scroll_behavior.scroll_direction_ratio = 
                            (scroll_behavior.scroll_direction_ratio * 0.7) + (direction_ratio * 0.3);
                        
                        // Calculate scroll pause duration
                        if mouse_data.scrolls.len() >= 2 {
                            let mut total_pause = 0;
                            let mut pause_count = 0;
                            
                            for i in 1..mouse_data.scrolls.len() {
                                let prev = &mouse_data.scrolls[i-1];
                                let curr = &mouse_data.scrolls[i];
                                
                                let pause_duration = curr.timestamp as i64 - prev.timestamp as i64;
                                if pause_duration > 0 && pause_duration < 2000 { // Ignore long pauses
                                    total_pause += pause_duration as u64;
                                    pause_count += 1;
                                }
                            }
                            
                            if pause_count > 0 {
                                let avg_pause = total_pause as f64 / pause_count as f64;
                                
                                // Update pause duration with weighted average
                                scroll_behavior.scroll_pause_duration = 
                                    (scroll_behavior.scroll_pause_duration * 0.7) + (avg_pause * 0.3);
                            }
                        }
                    }
                    
                    mouse_patterns.last_updated = Utc::now();
                }
            },
            ObservationType::SessionBehavior => {
                if let Some(session_data) = &observation.session_data {
                    let session_patterns = &mut profile.session_patterns;
                    
                    // Update session time distribution
                    if let Some(end_time) = session_data.end_time {
                        let hour = end_time.format("%H").to_string();
                        let entry = session_patterns.session_time_distribution
                            .entry(hour)
                            .or_insert(0.0);
                        *entry += 1.0;
                        
                        // Normalize distribution
                        let total = session_patterns.session_time_distribution.values().sum::<f64>();
                        for value in session_patterns.session_time_distribution.values_mut() {
                            *value /= total;
                        }
                        
                        // Update typical days
                        let day = end_time.format("%A").to_string();
                        let entry = session_patterns.typical_days
                            .entry(day)
                            .or_insert(0.0);
                        *entry += 1.0;
                        
                        // Normalize days
                        let total = session_patterns.typical_days.values().sum::<f64>();
                        for value in session_patterns.typical_days.values_mut() {
                            *value /= total;
                        }
                        
                        // Update session duration
                        let duration_minutes = (end_time - session_data.start_time).num_minutes() as f64;
                        
                        // Update with weighted average
                        session_patterns.typical_session_duration = 
                            (session_patterns.typical_session_duration * 0.7) + (duration_minutes * 0.3);
                    }
                    
                    // Update typical locations
                    if let Some(location) = &session_data.location {
                        let location_index = session_patterns.typical_locations.iter()
                            .position(|l| l.country == location.country 
                                   && l.region == location.region 
                                   && l.city == location.city);
                        
                        if let Some(index) = location_index {
                            // Update existing location
                            let loc = &mut session_patterns.typical_locations[index];
                            loc.frequency += 0.1;
                            loc.last_seen = Utc::now();
                        } else {
                            // Add new location
                            let mut new_location = location.clone();
                            new_location.frequency = 0.1;
                            new_location.first_seen = Utc::now();
                            new_location.last_seen = Utc::now();
                            session_patterns.typical_locations.push(new_location);
                        }
                        
                        // Normalize frequencies
                        let total = session_patterns.typical_locations.iter()
                            .map(|l| l.frequency)
                            .sum::<f64>();
                        
                        for location in &mut session_patterns.typical_locations {
                            location.frequency /= total;
                        }
                    }
                    
                    // Update average actions per session
                    if !session_data.session_actions.is_empty() {
                        let action_count = session_data.session_actions.len() as f64;
                        
                        // Update with weighted average
                        session_patterns.average_actions_per_session = 
                            (session_patterns.average_actions_per_session * 0.7) + (action_count * 0.3);
                    }
                    
                    // Update typical session flow
                    if session_data.session_actions.len() >= 2 {
                        // Create flow pattern from sequential page visits
                        let flow = session_data.session_actions.iter()
                            .map(|a| a.page.clone())
                            .collect::<Vec<String>>()
                            .join(" > ");
                        
                        // Add to typical flows
                        if !session_patterns.typical_session_flow.contains(&flow) {
                            session_patterns.typical_session_flow.push(flow);
                            
                            // Keep only the most recent 10 flows
                            if session_patterns.typical_session_flow.len() > 10 {
                                session_patterns.typical_session_flow.remove(0);
                            }
                        }
                    }
                    
                    session_patterns.last_updated = Utc::now();
                }
            },
            ObservationType::TransactionBehavior => {
                if let Some(transaction_data) = &observation.transaction_data {
                    let transaction_patterns = &mut profile.transaction_patterns;
                    
                    // Update typical transaction times
                    let hour = Utc::now().format("%H").to_string();
                    let entry = transaction_patterns.typical_transaction_times
                        .entry(hour)
                        .or_insert(0.0);
                    *entry += 1.0;
                    
                    // Normalize times
                    let total = transaction_patterns.typical_transaction_times.values().sum::<f64>();
                    for value in transaction_patterns.typical_transaction_times.values_mut() {
                        *value /= total;
                    }
                    
                    // Update typical assets
                    let entry = transaction_patterns.typical_assets
                        .entry(transaction_data.asset.clone())
                        .or_insert(0.0);
                    *entry += 1.0;
                    
                    // Normalize assets
                    let total = transaction_patterns.typical_assets.values().sum::<f64>();
                    for value in transaction_patterns.typical_assets.values_mut() {
                        *value /= total;
                    }
                    
                    // Update typical transaction amounts
                    let amount = transaction_data.amount;
                    let range_found = transaction_patterns.typical_transaction_amounts.iter()
                        .position(|r| amount >= r.min && (r.max == 0.0 || amount <= r.max));
                    
                    if let Some(index) = range_found {
                        // Update existing range
                        let range = &mut transaction_patterns.typical_transaction_amounts[index];
                        range.frequency += 0.1;
                    } else {
                        // Create new range
                        // Find appropriate range bounds based on amount
                        let (min, max) = if amount < 10.0 {
                            (0.0, 10.0)
                        } else if amount < 100.0 {
                            (10.0, 100.0)
                        } else if amount < 1000.0 {
                            (100.0, 1000.0)
                        } else if amount < 10000.0 {
                            (1000.0, 10000.0)
                        } else {
                            (10000.0, 0.0) // No upper limit
                        };
                        
                        transaction_patterns.typical_transaction_amounts.push(AmountRange {
                            min,
                            max,
                            frequency: 0.1,
                        });
                    }
                    
                    // Normalize amount ranges
                    let total = transaction_patterns.typical_transaction_amounts.iter()
                        .map(|r| r.frequency)
                        .sum::<f64>();
                    
                    for range in &mut transaction_patterns.typical_transaction_amounts {
                        range.frequency /= total;
                    }
                    
                    // Update specific patterns based on transaction type
                    match transaction_data.transaction_type {
                        TransactionType::Withdrawal => {
                            // Update withdrawal frequency
                            transaction_patterns.typical_withdrawal_frequency += 0.1;
                            
                            // Update typical withdrawal addresses
                            if let Some(destination) = &transaction_data.destination {
                                let entry = transaction_patterns.typical_withdrawal_addresses
                                    .entry(destination.clone())
                                    .or_insert(0.0);
                                *entry += 1.0;
                                
                                // Normalize addresses
                                let total = transaction_patterns.typical_withdrawal_addresses.values().sum::<f64>();
                                for value in transaction_patterns.typical_withdrawal_addresses.values_mut() {
                                    *value /= total;
                                }
                            }
                        },
                        TransactionType::Deposit => {
                            // Update typical deposit sources
                            if let Some(source) = &transaction_data.source {
                                let entry = transaction_patterns.typical_deposit_sources
                                    .entry(source.clone())
                                    .or_insert(0.0);
                                *entry += 1.0;
                                
                                // Normalize sources
                                let total = transaction_patterns.typical_deposit_sources.values().sum::<f64>();
                                for value in transaction_patterns.typical_deposit_sources.values_mut() {
                                    *value /= total;
                                }
                            }
                        },
                        TransactionType::Trade => {
                            // Update typical trading pairs
                            let pair = format!("{}/USD", transaction_data.asset); // Simplified
                            let entry = transaction_patterns.typical_trading_pairs
                                .entry(pair)
                                .or_insert(0.0);
                            *entry += 1.0;
                            
                            // Normalize pairs
                            let total = transaction_patterns.typical_trading_pairs.values().sum::<f64>();
                            for value in transaction_patterns.typical_trading_pairs.values_mut() {
                                *value /= total;
                            }
                        },
                        _ => {
                            // Other transaction types
                        }
                    }
                    
                    transaction_patterns.last_updated = Utc::now();
                }
            },
        }
        
        // Update profile status if still in learning phase
        if profile.status == ProfileStatus::Learning {
            // Count observations for this user
            let observation_count = {
                let observations = self.observations.read().unwrap();
                observations.iter()
                    .filter(|o| o.user_id == profile.user_id)
                    .count()
            };
            
            // Check if we have enough observations to exit learning phase
            let threshold = self.thresholds.read().unwrap().learning_phase_observations as usize;
            if observation_count >= threshold {
                profile.status = ProfileStatus::Active;
            }
        }
        
        // Update profile confidence score
        self.update_confidence_score(profile);
        
        profile.updated_at = Utc::now();
        
        Ok(())
    }
    
    // Update profile confidence score
    fn update_confidence_score(&self, profile: &mut UserBehaviorProfile) {
        // Calculate confidence based on available patterns
        let mut components = Vec::new();
        let mut weights = Vec::new();
        
        // Confidence from device fingerprints
        if !profile.device_fingerprints.is_empty() {
            components.push(0.7); // Base confidence from having device data
            weights.push(0.3);    // Weight for device data
        }
        
        // Confidence from typing patterns
        if let Some(typing) = &profile.typing_patterns {
            let typing_confidence = if !typing.typing_rhythm.is_empty() && !typing.key_down_duration.is_empty() {
                0.8 // High confidence from strong typing data
            } else {
                0.4 // Lower confidence from partial typing data
            };
            
            components.push(typing_confidence);
            weights.push(0.2); // Weight for typing data
        }
        
        // Confidence from mouse patterns
        if let Some(mouse) = &profile.mouse_patterns {
            let mouse_confidence = if !mouse.click_patterns.is_empty() && mouse.cursor_path_straightness > 0.0 {
                0.7 // Good confidence from mouse data
            } else {
                0.3 // Lower confidence from partial mouse data
            };
            
            components.push(mouse_confidence);
            weights.push(0.15); // Weight for mouse data
        }
        
        // Confidence from session patterns
        if profile.session_patterns.typical_session_duration > 0.0 
           && !profile.session_patterns.typical_locations.is_empty() {
            let session_confidence = if !profile.session_patterns.typical_session_flow.is_empty() {
                0.85 // High confidence from detailed session data
            } else {
                0.6  // Moderate confidence from basic session data
            };
            
            components.push(session_confidence);
            weights.push(0.2); // Weight for session data
        }
        
        // Confidence from transaction patterns
        if !profile.transaction_patterns.typical_transaction_amounts.is_empty() 
           && !profile.transaction_patterns.typical_assets.is_empty() {
            let transaction_confidence = if !profile.transaction_patterns.typical_withdrawal_addresses.is_empty() {
                0.9 // High confidence from detailed transaction history
            } else {
                0.6 // Moderate confidence from basic transaction data
            };
            
            components.push(transaction_confidence);
            weights.push(0.25); // Weight for transaction data
        }
        
        // Calculate weighted average
        if !components.is_empty() {
            let total_weight: f64 = weights.iter().sum();
            let weighted_sum: f64 = components.iter().zip(weights.iter())
                .map(|(c, w)| c * w)
                .sum();
            
            profile.confidence_score = weighted_sum / total_weight;
        } else {
            profile.confidence_score = 0.0;
        }
    }
    
    // Authenticate a user using behavioral biometrics
    pub async fn authenticate_user(
        &self,
        user_id: &str,
        session_id: &str,
        observations: &[BehaviorObservation],
    ) -> Result<AuthenticationResult, String> {
        // Get user profile
        let profile_id = {
            let profile_by_user = self.profile_by_user.read().unwrap();
            profile_by_user.get(user_id)
                .cloned()
                .ok_or_else(|| "User profile not found".to_string())?
        };
        
        let profile = {
            let profiles = self.profiles.read().unwrap();
            profiles.get(&profile_id)
                .cloned()
                .ok_or_else(|| "Profile not found".to_string())?
        };
        
        // If profile is still in learning phase, always allow
        if profile.status == ProfileStatus::Learning {
            return Ok(AuthenticationResult {
                user_id: user_id.to_string(),
                session_id: session_id.to_string(),
                timestamp: Utc::now(),
                authenticated: true,
                confidence_score: profile.confidence_score,
                risk_score: 0.2, // Low risk during learning phase
                anomaly_detected: false,
                anomalous_features: Vec::new(),
                auth_factors_used: vec!["behavioral".to_string()],
                recommendation: AuthRecommendation::Allow,
            });
        }
        
        // If profile is locked or suspended, block authentication
        if profile.status == ProfileStatus::Locked || profile.status == ProfileStatus::Suspended {
            return Ok(AuthenticationResult {
                user_id: user_id.to_string(),
                session_id: session_id.to_string(),
                timestamp: Utc::now(),
                authenticated: false,
                confidence_score: profile.confidence_score,
                risk_score: 1.0, // Maximum risk for locked profiles
                anomaly_detected: true,
                anomalous_features: vec!["profile_status".to_string()],
                auth_factors_used: vec!["behavioral".to_string()],
                recommendation: AuthRecommendation::Block,
            });
        }
        
        // Calculate authentication scores for each observation type
        let thresholds = self.thresholds.read().unwrap();
        let mut scores = Vec::new();
        let mut anomalous_features = Vec::new();
        
        // Process typing observations
        let typing_observations: Vec<&BehaviorObservation> = observations.iter()
            .filter(|o| o.observation_type == ObservationType::TypingBehavior)
            .collect();
        
        if !typing_observations.is_empty() && profile.typing_patterns.is_some() {
            let typing_score = self.calculate_typing_match_score(&profile, &typing_observations)?;
            scores.push((typing_score, thresholds.feature_weights.typing_weight));
            
            if typing_score < thresholds.typing_match_threshold {
                anomalous_features.push("typing_pattern".to_string());
            }
        }
        
        // Process mouse observations
        let mouse_observations: Vec<&BehaviorObservation> = observations.iter()
            .filter(|o| o.observation_type == ObservationType::MouseBehavior)
            .collect();
        
        if !mouse_observations.is_empty() && profile.mouse_patterns.is_some() {
            let mouse_score = self.calculate_mouse_match_score(&profile, &mouse_observations)?;
            scores.push((mouse_score, thresholds.feature_weights.mouse_weight));
            
            if mouse_score < thresholds.mouse_match_threshold {
                anomalous_features.push("mouse_pattern".to_string());
            }
        }
        
        // Process session observations
        let session_observations: Vec<&BehaviorObservation> = observations.iter()
            .filter(|o| o.observation_type == ObservationType::SessionBehavior)
            .collect();
        
        if !session_observations.is_empty() {
            let session_score = self.calculate_session_match_score(&profile, &session_observations)?;
            scores.push((session_score, thresholds.feature_weights.session_weight));
            
            if session_score < thresholds.session_match_threshold {
                anomalous_features.push("session_pattern".to_string());
            }
        }
        
        // Process device fingerprint observations
        let device_observations: Vec<&BehaviorObservation> = observations.iter()
            .filter(|o| o.observation_type == ObservationType::DeviceFingerprint)
            .collect();
        
        if !device_observations.is_empty() {
            let device_score = self.calculate_device_match_score(&profile, &device_observations)?;
            scores.push((device_score, thresholds.feature_weights.device_weight));
            
            if device_score < 0.5 { // Custom threshold for device matching
                anomalous_features.push("unknown_device".to_string());
            }
        }
        
        // Calculate overall score
        let total_weight: f64 = scores.iter().map(|(_, w)| w).sum();
        let overall_score: f64 = if total_weight > 0.0 {
            scores.iter().map(|(s, w)| s * w).sum::<f64>() / total_weight
        } else {
            0.0
        };
        
        // Calculate risk score (inverse of the match score with adjustments)
        let risk_score = 1.0 - (overall_score * 0.8);
        
        // Determine authentication result
        let (authenticated, recommendation) = if overall_score >= thresholds.overall_match_threshold {
            if risk_score > thresholds.high_risk_threshold {
                // High risk despite good match - require additional verification
                (true, AuthRecommendation::AdditionalFactorRequired)
            } else {
                // Good match, acceptable risk
                (true, AuthRecommendation::Allow)
            }
        } else if overall_score >= thresholds.overall_match_threshold * 0.7 {
            // Borderline match - require additional verification
            (true, AuthRecommendation::AdditionalFactorRequired)
        } else if overall_score >= thresholds.overall_match_threshold * 0.5 {
            // Poor match - require manual review
            (false, AuthRecommendation::ManualReview)
        } else {
            // Very poor match - block
            (false, AuthRecommendation::Block)
        };
        
        // Create authentication result
        let result = AuthenticationResult {
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            timestamp: Utc::now(),
            authenticated,
            confidence_score: overall_score,
            risk_score,
            anomaly_detected: !anomalous_features.is_empty(),
            anomalous_features,
            auth_factors_used: vec!["behavioral".to_string()],
            recommendation,
        };
        
        // Log high-risk authentication attempts
        if risk_score > thresholds.high_risk_threshold {
            self.security_monitoring.log_event(
                "HighRiskAuthentication",
                Some(user_id),
                None,
                Some(&profile_id),
                "High",
                serde_json::json!({
                    "session_id": session_id,
                    "risk_score": risk_score,
                    "confidence_score": overall_score,
                    "anomalous_features": result.anomalous_features,
                    "recommendation": format!("{:?}", result.recommendation),
                }),
            ).await;
        }
        
        Ok(result)
    }
    
    // Calculate typing match score
    async fn calculate_typing_anomaly_score(
        &self,
        profile_id: &str,
        typing_data: &TypingData,
    ) -> Result<f64, String> {
        let profiles = self.profiles.read().unwrap();
        let profile = profiles.get(profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        // If no typing patterns available, return moderate anomaly score
        if profile.typing_patterns.is_none() {
            return Ok(0.5);
        }
        
        let typing_patterns = profile.typing_patterns.as_ref().unwrap();
        
        // Calculate anomaly components
        let mut anomaly_scores = Vec::new();
        
        // Typing speed anomaly
        if typing_patterns.average_typing_speed > 0.0 {
            let char_count = typing_data.keystroke_times.len() as f64;
            let duration_minutes = typing_data.typing_duration as f64 / 60000.0;
            let typing_speed = if duration_minutes > 0.0 {
                char_count / duration_minutes
            } else {
                0.0
            };
            
            let speed_diff = (typing_speed - typing_patterns.average_typing_speed).abs();
            let speed_anomaly = (speed_diff / typing_patterns.average_typing_speed).min(1.0);
            
            anomaly_scores.push(speed_anomaly);
        }
        
        // Key down duration anomaly
        if !typing_patterns.key_down_duration.is_empty() {
            let mut duration_diffs = Vec::new();
            
            for keystroke in &typing_data.keystroke_times {
                if let Some(expected_duration) = typing_patterns.key_down_duration.get(&keystroke.key) {
                    let actual_duration = (keystroke.key_up_time - keystroke.key_down_time) as f64;
                    let diff = (actual_duration - *expected_duration).abs();
                    let relative_diff = if *expected_duration > 0.0 {
                        diff / *expected_duration
                    } else {
                        1.0
                    };
                    
                    duration_diffs.push(relative_diff.min(1.0));
                }
            }
            
            if !duration_diffs.is_empty() {
                let avg_duration_anomaly = duration_diffs.iter().sum::<f64>() / duration_diffs.len() as f64;
                anomaly_scores.push(avg_duration_anomaly);
            }
        }
        
        // Error rate anomaly
        if typing_patterns.error_rate > 0.0 {
            let error_rate = if typing_data.keystroke_times.len() > 0 {
                typing_data.error_count as f64 / typing_data.keystroke_times.len() as f64
            } else {
                0.0
            };
            
            let error_diff = (error_rate - typing_patterns.error_rate).abs();
            let error_anomaly = if typing_patterns.error_rate > 0.0 {
                (error_diff / typing_patterns.error_rate).min(1.0)
            } else {
                error_rate.min(1.0)
            };
            
            anomaly_scores.push(error_anomaly);
        }
        
        // Backspace usage anomaly
        if typing_patterns.backspace_usage > 0.0 {
            let backspace_rate = if typing_data.keystroke_times.len() > 0 {
                typing_data.backspace_count as f64 / typing_data.keystroke_times.len() as f64
            } else {
                0.0
            };
            
            let backspace_diff = (backspace_rate - typing_patterns.backspace_usage).abs();
            let backspace_anomaly = if typing_patterns.backspace_usage > 0.0 {
                (backspace_diff / typing_patterns.backspace_usage).min(1.0)
            } else {
                backspace_rate.min(1.0)
            };
            
            anomaly_scores.push(backspace_anomaly);
        }
        
        // Typing rhythm anomaly
        if typing_data.keystroke_times.len() >= 2 && !typing_patterns.typing_rhythm.is_empty() {
            let mut inter_key_times = Vec::new();
            
            for i in 1..typing_data.keystroke_times.len() {
                let prev = &typing_data.keystroke_times[i-1];
                let curr = &typing_data.keystroke_times[i];
                
                let time_diff = curr.key_down_time as i64 - prev.key_up_time as i64;
                if time_diff > 0 && time_diff < 1000 { // Filter out pauses
                    inter_key_times.push(time_diff as f64);
                }
            }
            
            if !inter_key_times.is_empty() {
                // Compare rhythm patterns using dynamic time warping or similar technique
                // For simplicity, we'll use a basic average comparison
                let observed_avg = inter_key_times.iter().sum::<f64>() / inter_key_times.len() as f64;
                let expected_avg = typing_patterns.typing_rhythm.iter().sum::<f64>() / typing_patterns.typing_rhythm.len() as f64;
                
                let rhythm_diff = (observed_avg - expected_avg).abs();
                let rhythm_anomaly = if expected_avg > 0.0 {
                    (rhythm_diff / expected_avg).min(1.0)
                } else {
                    0.5
                };
                
                anomaly_scores.push(rhythm_anomaly);
            }
        }
        
        // Bigram timing anomaly
        if typing_data.keystroke_times.len() >= 2 && !typing_patterns.common_bigrams.is_empty() {
            let mut bigram_diffs = Vec::new();
            
            for i in 1..typing_data.keystroke_times.len() {
                let prev = &typing_data.keystroke_times[i-1];
                let curr = &typing_data.keystroke_times[i];
                
                let bigram = format!("{}{}", prev.key, curr.key);
                let time_diff = curr.key_down_time as i64 - prev.key_down_time as i64;
                
                if time_diff > 0 && time_diff < 1000 { // Filter out pauses
                    if let Some(expected_time) = typing_patterns.common_bigrams.get(&bigram) {
                        let diff = ((time_diff as f64) - expected_time).abs();
                        let relative_diff = if *expected_time > 0.0 {
                            diff / *expected_time
                        } else {
                            1.0
                        };
                        
                        bigram_diffs.push(relative_diff.min(1.0));
                    }
                }
            }
            
            if !bigram_diffs.is_empty() {
                let avg_bigram_anomaly = bigram_diffs.iter().sum::<f64>() / bigram_diffs.len() as f64;
                anomaly_scores.push(avg_bigram_anomaly);
            }
        }
        
        // Calculate overall anomaly score
        if anomaly_scores.is_empty() {
            Ok(0.5) // Default moderate anomaly if no comparison possible
        } else {
            let overall_anomaly = anomaly_scores.iter().sum::<f64>() / anomaly_scores.len() as f64;
            Ok(overall_anomaly)
        }
    }
    
    // Calculate mouse anomaly score
    async fn calculate_mouse_anomaly_score(
        &self,
        profile_id: &str,
        mouse_data: &MouseData,
    ) -> Result<f64, String> {
        let profiles = self.profiles.read().unwrap();
        let profile = profiles.get(profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        // If no mouse patterns available, return moderate anomaly score
        if profile.mouse_patterns.is_none() {
            return Ok(0.5);
        }
        
        let mouse_patterns = profile.mouse_patterns.as_ref().unwrap();
        
        // Calculate anomaly components
        let mut anomaly_scores = Vec::new();
        
        // Mouse speed anomaly
        if mouse_patterns.average_speed > 0.0 && mouse_data.total_duration > 0 {
            let speed = mouse_data.total_distance / (mouse_data.total_duration as f64 / 1000.0);
            let speed_diff = (speed - mouse_patterns.average_speed).abs();
            let speed_anomaly = if mouse_patterns.average_speed > 0.0 {
                (speed_diff / mouse_patterns.average_speed).min(1.0)
            } else {
                0.5
            };
            
            anomaly_scores.push(speed_anomaly);
        }
        
        // Click pattern anomaly
        if !mouse_patterns.click_patterns.is_empty() && !mouse_data.mouse_clicks.is_empty() {
            let mut click_context_counts = HashMap::new();
            
            // Count clicks by context
            for click in &mouse_data.mouse_clicks {
                let context_key = format!("{}:{}", click.element_type, click.context);
                let entry = click_context_counts.entry(context_key).or_insert(0);
                *entry += 1;
            }
            
            // Convert to frequency distribution
            let total_clicks = mouse_data.mouse_clicks.len() as f64;
            let mut click_distribution = HashMap::new();
            
            for (context, count) in click_context_counts {
                click_distribution.insert(context, count as f64 / total_clicks);
            }
            
            // Calculate Jensen-Shannon divergence between distributions
            let mut js_divergence = 0.0;
            let mut matched_contexts = 0;
            
            for (context, expected_freq) in &mouse_patterns.click_patterns {
                if let Some(observed_freq) = click_distribution.get(context) {
                    // Calculate KL divergence component
                    if *expected_freq > 0.0 && *observed_freq > 0.0 {
                        let m = (expected_freq + observed_freq) * 0.5;
                        let kl1 = expected_freq * (expected_freq / m).ln();
                        let kl2 = observed_freq * (observed_freq / m).ln();
                        js_divergence += (kl1 + kl2) * 0.5;
                    }
                    matched_contexts += 1;
                }
            }
            
            // Normalize and convert to anomaly score
            if matched_contexts > 0 {
                js_divergence /= matched_contexts as f64;
                let click_anomaly = (1.0 - (-js_divergence).exp()).min(1.0);
                anomaly_scores.push(click_anomaly);
            }
        }
        
        // Cursor path straightness anomaly
        if mouse_patterns.cursor_path_straightness > 0.0 && mouse_data.mouse_movements.len() >= 2 {
            let mut total_actual_distance = 0.0;
            let mut total_direct_distance = 0.0;
            
            for i in 1..mouse_data.mouse_movements.len() {
                let prev = &mouse_data.mouse_movements[i-1];
                let curr = &mouse_data.mouse_movements[i];
                
                let actual_distance = (
                    ((curr.x as f64 - prev.x as f64).powi(2) + 
                     (curr.y as f64 - prev.y as f64).powi(2)).sqrt()
                );
                
                total_actual_distance += actual_distance;
            }
            
            let first = &mouse_data.mouse_movements.first().unwrap();
            let last = &mouse_data.mouse_movements.last().unwrap();
            
            total_direct_distance = (
                ((last.x as f64 - first.x as f64).powi(2) + 
                 (last.y as f64 - first.y as f64).powi(2)).sqrt()
            );
            
            if total_actual_distance > 0.0 {
                let straightness = total_direct_distance / total_actual_distance;
                let expected_straightness = mouse_patterns.cursor_path_straightness;
                
                let straightness_diff = (straightness - expected_straightness).abs();
                let straightness_anomaly = if expected_straightness > 0.0 {
                    (straightness_diff / expected_straightness).min(1.0)
                } else {
                    0.5
                };
                
                anomaly_scores.push(straightness_anomaly);
            }
        }
        
        // Scroll behavior anomaly
        if !mouse_data.scrolls.is_empty() {
            let scroll_behavior = &mouse_patterns.scroll_behavior;
            
            if scroll_behavior.average_scroll_speed > 0.0 {
                let total_scroll_speed: f64 = mouse_data.scrolls.iter()
                    .map(|s| s.speed)
                    .sum();
                
                let avg_scroll_speed = total_scroll_speed / mouse_data.scrolls.len() as f64;
                let speed_diff = (avg_scroll_speed - scroll_behavior.average_scroll_speed).abs();
                
                let speed_anomaly = if scroll_behavior.average_scroll_speed > 0.0 {
                    (speed_diff / scroll_behavior.average_scroll_speed).min(1.0)
                } else {
                    0.5
                };
                
                anomaly_scores.push(speed_anomaly);
            }
            
            // Direction ratio anomaly
            let down_scrolls = mouse_data.scrolls.iter()
                .filter(|s| s.delta_y > 0)
                .count() as f64;
            
            let direction_ratio = down_scrolls / mouse_data.scrolls.len() as f64;
            let ratio_diff = (direction_ratio - scroll_behavior.scroll_direction_ratio).abs();
            
            // Direction ratio is always 0.0-1.0 range, so simple difference works
            anomaly_scores.push(ratio_diff);
        }
        
        // Calculate overall anomaly score
        if anomaly_scores.is_empty() {
            Ok(0.5) // Default moderate anomaly if no comparison possible
        } else {
            let overall_anomaly = anomaly_scores.iter().sum::<f64>() / anomaly_scores.len() as f64;
            Ok(overall_anomaly)
        }
    }
    
    // Calculate session anomaly score
    async fn calculate_session_anomaly_score(
        &self,
        profile_id: &str,
        session_data: &SessionData,
    ) -> Result<f64, String> {
        let profiles = self.profiles.read().unwrap();
        let profile = profiles.get(profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        let session_patterns = &profile.session_patterns;
        
        // Calculate anomaly components
        let mut anomaly_scores = Vec::new();
        
        // Location anomaly
        if let Some(location) = &session_data.location {
            // Check if location is in typical locations
            let location_match = session_patterns.typical_locations.iter()
                .find(|l| l.country == location.country 
                      && l.region == location.region 
                      && l.city == location.city);
            
            let location_anomaly = if let Some(match_loc) = location_match {
                // Inverse of frequency (less frequent = higher anomaly)
                1.0 - match_loc.frequency
            } else {
                // New location is high anomaly
                0.9
            };
            
            anomaly_scores.push(location_anomaly);
        }
        
        // Time pattern anomaly
        if !session_patterns.session_time_distribution.is_empty() {
            let hour = Utc::now().format("%H").to_string();
            let hour_frequency = session_patterns.session_time_distribution
                .get(&hour)
                .cloned()
                .unwrap_or(0.0);
            
            // Inverse of frequency (less frequent = higher anomaly)
            let time_anomaly = 1.0 - hour_frequency;
            anomaly_scores.push(time_anomaly);
        }
        
        // Day pattern anomaly
        if !session_patterns.typical_days.is_empty() {
            let day = Utc::now().format("%A").to_string();
            let day_frequency = session_patterns.typical_days
                .get(&day)
                .cloned()
                .unwrap_or(0.0);
            
            // Inverse of frequency (less frequent = higher anomaly)
            let day_anomaly = 1.0 - day_frequency;
            anomaly_scores.push(day_anomaly);
        }
        
        // Device anomaly
        let device_match = profile.device_fingerprints.iter()
            .find(|d| d.id == session_data.device_id);
        
        let device_anomaly = if device_match.is_some() {
            // Known device, check usage frequency
            let device_frequency = session_patterns.device_usage_ratios
                .get(&session_data.device_id)
                .cloned()
                .unwrap_or(0.0);
            
            // Inverse of frequency (less frequent = higher anomaly)
            1.0 - device_frequency
        } else {
            // Unknown device is high anomaly
            0.95
        };
        
        anomaly_scores.push(device_anomaly);
        
        // Session flow anomaly
        if !session_patterns.typical_session_flow.is_empty() && !session_data.session_actions.is_empty() {
            // Create flow pattern from sequential page visits
            let flow = session_data.session_actions.iter()
                .map(|a| a.page.clone())
                .collect::<Vec<String>>()
                .join(" > ");
            
            // Check if flow matches any typical flows
            let flow_match = session_patterns.typical_session_flow.iter()
                .any(|f| f.contains(&flow) || flow.contains(f));
            
            let flow_anomaly = if flow_match {
                0.2 // Low anomaly for matching flow
            } else {
                0.7 // High anomaly for non-matching flow
            };
            
            anomaly_scores.push(flow_anomaly);
        }
        
        // Calculate overall anomaly score
        if anomaly_scores.is_empty() {
            Ok(0.5) // Default moderate anomaly if no comparison possible
        } else {
            let overall_anomaly = anomaly_scores.iter().sum::<f64>() / anomaly_scores.len() as f64;
            Ok(overall_anomaly)
        }
    }
    
    // Calculate transaction anomaly score
    async fn calculate_transaction_anomaly_score(
        &self,
        profile_id: &str,
        transaction_data: &TransactionData,
    ) -> Result<f64, String> {
        let profiles = self.profiles.read().unwrap();
        let profile = profiles.get(profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        let transaction_patterns = &profile.transaction_patterns;
        
        // Calculate anomaly components
        let mut anomaly_scores = Vec::new();
        
        // Amount anomaly
        if !transaction_patterns.typical_transaction_amounts.is_empty() {
            let amount = transaction_data.amount;
            
            // Find matching amount range
            let amount_range = transaction_patterns.typical_transaction_amounts.iter()
                .find(|r| amount >= r.min && (r.max == 0.0 || amount <= r.max));
            
            let amount_anomaly = if let Some(range) = amount_range {
                // Inverse of frequency (less frequent = higher anomaly)
                1.0 - range.frequency
            } else {
                // Amount outside typical ranges is high anomaly
                0.9
            };
            
            anomaly_scores.push(amount_anomaly);
        }
        
        // Asset anomaly
        if !transaction_patterns.typical_assets.is_empty() {
            let asset_frequency = transaction_patterns.typical_assets
                .get(&transaction_data.asset)
                .cloned()
                .unwrap_or(0.0);
            
            // Inverse of frequency (less frequent = higher anomaly)
            let asset_anomaly = 1.0 - asset_frequency;
            anomaly_scores.push(asset_anomaly);
        }
        
        // Time pattern anomaly
        if !transaction_patterns.typical_transaction_times.is_empty() {
            let hour = transaction_data.timestamp.format("%H").to_string();
            let hour_frequency = transaction_patterns.typical_transaction_times
                .get(&hour)
                .cloned()
                .unwrap_or(0.0);
            
            // Inverse of frequency (less frequent = higher anomaly)
            let time_anomaly = 1.0 - hour_frequency;
            anomaly_scores.push(time_anomaly);
        }
        
        // Transaction-type specific anomalies
        match transaction_data.transaction_type {
            TransactionType::Withdrawal => {
                // Withdrawal address anomaly
                if !transaction_patterns.typical_withdrawal_addresses.is_empty() {
                    if let Some(destination) = &transaction_data.destination {
                        let address_frequency = transaction_patterns.typical_withdrawal_addresses
                            .get(destination)
                            .cloned()
                            .unwrap_or(0.0);
                        
                        // Inverse of frequency (less frequent = higher anomaly)
                        let address_anomaly = 1.0 - address_frequency;
                        
                        // Weight withdrawal address anomalies higher for security
                        anomaly_scores.push(address_anomaly);
                        anomaly_scores.push(address_anomaly); // Double weight
                    } else {
                        // Missing destination for withdrawal is highly anomalous
                        anomaly_scores.push(0.95);
                    }
                }
            },
            TransactionType::Deposit => {
                // Deposit source anomaly
                if !transaction_patterns.typical_deposit_sources.is_empty() {
                    if let Some(source) = &transaction_data.source {
                        let source_frequency = transaction_patterns.typical_deposit_sources
                            .get(source)
                            .cloned()
                            .unwrap_or(0.0);
                        
                        // Inverse of frequency (less frequent = higher anomaly)
                        let source_anomaly = 1.0 - source_frequency;
                        anomaly_scores.push(source_anomaly);
                    }
                }
            },
            TransactionType::Trade => {
                // Trading pair anomaly
                if !transaction_patterns.typical_trading_pairs.is_empty() {
                    let pair = format!("{}/USD", transaction_data.asset); // Simplified
                    let pair_frequency = transaction_patterns.typical_trading_pairs
                        .get(&pair)
                        .cloned()
                        .unwrap_or(0.0);
                    
                    // Inverse of frequency (less frequent = higher anomaly)
                    let pair_anomaly = 1.0 - pair_frequency;
                    anomaly_scores.push(pair_anomaly);
                }
            },
            _ => {
                // Other transaction types
            }
        }
        
        // Device anomaly
        let device_known = profile.device_fingerprints.iter()
            .any(|d| d.id == transaction_data.device_id);
        
        let device_anomaly = if device_known {
            0.2 // Low anomaly for known device
        } else {
            0.9 // High anomaly for unknown device
        };
        
        anomaly_scores.push(device_anomaly);
        
        // Calculate overall anomaly score
        if anomaly_scores.is_empty() {
            Ok(0.5) // Default moderate anomaly if no comparison possible
        } else {
            let overall_anomaly = anomaly_scores.iter().sum::<f64>() / anomaly_scores.len() as f64;
            Ok(overall_anomaly)
        }
    }
    
    // Calculate typing match score for authentication
    fn calculate_typing_match_score(
        &self,
        profile: &UserBehaviorProfile,
        observations: &[&BehaviorObservation],
    ) -> Result<f64, String> {
        let typing_patterns = profile.typing_patterns.as_ref()
            .ok_or_else(|| "No typing patterns available".to_string())?;
        
        let mut match_scores = Vec::new();
        
        for obs in observations {
            if let Some(typing_data) = &obs.typing_data {
                // Calculate typing speed match
                if typing_patterns.average_typing_speed > 0.0 {
                    let char_count = typing_data.keystroke_times.len() as f64;
                    let duration_minutes = typing_data.typing_duration as f64 / 60000.0;
                    let typing_speed = if duration_minutes > 0.0 {
                        char_count / duration_minutes
                    } else {
                        0.0
                    };
                    
                    let speed_diff = (typing_speed - typing_patterns.average_typing_speed).abs();
                    let speed_match = (1.0 - (speed_diff / typing_patterns.average_typing_speed).min(1.0))
                        .max(0.0);
                    
                    match_scores.push(speed_match);
                }
                
                // Calculate key down duration match
                if !typing_patterns.key_down_duration.is_empty() {
                    let mut duration_matches = Vec::new();
                    
                    for keystroke in &typing_data.keystroke_times {
                        if let Some(expected_duration) = typing_patterns.key_down_duration.get(&keystroke.key) {
                            let actual_duration = (keystroke.key_up_time - keystroke.key_down_time) as f64;
                            let diff = (actual_duration - *expected_duration).abs();
                            let relative_diff = if *expected_duration > 0.0 {
                                diff / *expected_duration
                            } else {
                                1.0
                            };
                            
                            duration_matches.push((1.0 - relative_diff.min(1.0)).max(0.0));
                        }
                    }
                    
                    if !duration_matches.is_empty() {
                        let avg_duration_match = duration_matches.iter().sum::<f64>() / duration_matches.len() as f64;
                        match_scores.push(avg_duration_match);
                    }
                }
                
                // Calculate error rate match
                if typing_patterns.error_rate > 0.0 {
                    let error_rate = if typing_data.keystroke_times.len() > 0 {
                        typing_data.error_count as f64 / typing_data.keystroke_times.len() as f64
                    } else {
                        0.0
                    };
                    
                    let error_diff = (error_rate - typing_patterns.error_rate).abs();
                    let error_match = if typing_patterns.error_rate > 0.0 {
                        (1.0 - (error_diff / typing_patterns.error_rate).min(1.0)).max(0.0)
                    } else {
                        (1.0 - error_rate).max(0.0)
                    };
                    
                    match_scores.push(error_match);
                }
                
                // Calculate typing rhythm match
                if typing_data.keystroke_times.len() >= 2 && !typing_patterns.typing_rhythm.is_empty() {
                    let mut inter_key_times = Vec::new();
                    
                    for i in 1..typing_data.keystroke_times.len() {
                        let prev = &typing_data.keystroke_times[i-1];
                        let curr = &typing_data.keystroke_times[i];
                        
                        let time_diff = curr.key_down_time as i64 - prev.key_up_time as i64;
                        if time_diff > 0 && time_diff < 1000 { // Filter out pauses
                            inter_key_times.push(time_diff as f64);
                        }
                    }
                    
                    if !inter_key_times.is_empty() {
                        // For simplicity, we'll use a basic average comparison
                        let observed_avg = inter_key_times.iter().sum::<f64>() / inter_key_times.len() as f64;
                        let expected_avg = typing_patterns.typing_rhythm.iter().sum::<f64>() / typing_patterns.typing_rhythm.len() as f64;
                        
                        let rhythm_diff = (observed_avg - expected_avg).abs();
                        let rhythm_match = if expected_avg > 0.0 {
                            (1.0 - (rhythm_diff / expected_avg).min(1.0)).max(0.0)
                        } else {
                            0.5
                        };
                        
                        match_scores.push(rhythm_match);
                    }
                }
            }
        }
        
        // Calculate overall match score
        if match_scores.is_empty() {
            Ok(0.0) // No match if no comparison possible
        } else {
            let overall_match = match_scores.iter().sum::<f64>() / match_scores.len() as f64;
            Ok(overall_match)
        }
    }
    
    // Calculate mouse match score for authentication
    fn calculate_mouse_match_score(
        &self,
        profile: &UserBehaviorProfile,
        observations: &[&BehaviorObservation],
    ) -> Result<f64, String> {
        let mouse_patterns = profile.mouse_patterns.as_ref()
            .ok_or_else(|| "No mouse patterns available".to_string())?;
        
        let mut match_scores = Vec::new();
        
        for obs in observations {
            if let Some(mouse_data) = &obs.mouse_data {
                // Calculate mouse speed match
                if mouse_patterns.average_speed > 0.0 && mouse_data.total_duration > 0 {
                    let speed = mouse_data.total_distance / (mouse_data.total_duration as f64 / 1000.0);
                    let speed_diff = (speed - mouse_patterns.average_speed).abs();
                    let speed_match = if mouse_patterns.average_speed > 0.0 {
                        (1.0 - (speed_diff / mouse_patterns.average_speed).min(1.0)).max(0.0)
                    } else {
                        0.5
                    };
                    
                    match_scores.push(speed_match);
                }
                
                // Calculate cursor path straightness match
                if mouse_patterns.cursor_path_straightness > 0.0 && mouse_data.mouse_movements.len() >= 2 {
                    let mut total_actual_distance = 0.0;
                    let mut total_direct_distance = 0.0;
                    
                    for i in 1..mouse_data.mouse_movements.len() {
                        let prev = &mouse_data.mouse_movements[i-1];
                        let curr = &mouse_data.mouse_movements[i];
                        
                        let actual_distance = (
                            ((curr.x as f64 - prev.x as f64).powi(2) + 
                             (curr.y as f64 - prev.y as f64).powi(2)).sqrt()
                        );
                        
                        total_actual_distance += actual_distance;
                    }
                    
                    let first = &mouse_data.mouse_movements.first().unwrap();
                    let last = &mouse_data.mouse_movements.last().unwrap();
                    
                    total_direct_distance = (
                        ((last.x as f64 - first.x as f64).powi(2) + 
                         (last.y as f64 - first.y as f64).powi(2)).sqrt()
                    );
                    
                    if total_actual_distance > 0.0 {
                        let straightness = total_direct_distance / total_actual_distance;
                        let expected_straightness = mouse_patterns.cursor_path_straightness;
                        
                        let straightness_diff = (straightness - expected_straightness).abs();
                        let straightness_match = if expected_straightness > 0.0 {
                            (1.0 - (straightness_diff / expected_straightness).min(1.0)).max(0.0)
                        } else {
                            0.5
                        };
                        
                        match_scores.push(straightness_match);
                    }
                }
                
                // Calculate scroll behavior match
                if !mouse_data.scrolls.is_empty() {
                    let scroll_behavior = &mouse_patterns.scroll_behavior;
                    
                    if scroll_behavior.average_scroll_speed > 0.0 {
                        let total_scroll_speed: f64 = mouse_data.scrolls.iter()
                            .map(|s| s.speed)
                            .sum();
                        
                        let avg_scroll_speed = total_scroll_speed / mouse_data.scrolls.len() as f64;
                        let speed_diff = (avg_scroll_speed - scroll_behavior.average_scroll_speed).abs();
                        
                        let speed_match = if scroll_behavior.average_scroll_speed > 0.0 {
                            (1.0 - (speed_diff / scroll_behavior.average_scroll_speed).min(1.0)).max(0.0)
                        } else {
                            0.5
                        };
                        
                        match_scores.push(speed_match);
                    }
                    
                    // Direction ratio match
                    let down_scrolls = mouse_data.scrolls.iter()
                        .filter(|s| s.delta_y > 0)
                        .count() as f64;
                    
                    let direction_ratio = down_scrolls / mouse_data.scrolls.len() as f64;
                    let ratio_diff = (direction_ratio - scroll_behavior.scroll_direction_ratio).abs();
                    
                    // Direction ratio is always 0.0-1.0 range, so simple difference works
                    let ratio_match = (1.0 - ratio_diff).max(0.0);
                    match_scores.push(ratio_match);
                }
            }
        }
        
        // Calculate overall match score
        if match_scores.is_empty() {
            Ok(0.0) // No match if no comparison possible
        } else {
            let overall_match = match_scores.iter().sum::<f64>() / match_scores.len() as f64;
            Ok(overall_match)
        }
    }
    
    // Calculate session match score for authentication
    fn calculate_session_match_score(
        &self,
        profile: &UserBehaviorProfile,
        observations: &[&BehaviorObservation],
    ) -> Result<f64, String> {
        let session_patterns = &profile.session_patterns;
        
        let mut match_scores = Vec::new();
        
        for obs in observations {
            if let Some(session_data) = &obs.session_data {
                // Location match
                if let Some(location) = &session_data.location {
                    // Check if location is in typical locations
                    let location_match = session_patterns.typical_locations.iter()
                        .find(|l| l.country == location.country 
                             && l.region == location.region 
                             && l.city == location.city);
                    
                    let location_score = if let Some(match_loc) = location_match {
                        // Direct match with frequency as score
                        match_loc.frequency
                    } else {
                        // New location has low match score
                        0.1
                    };
                    
                    match_scores.push(location_score);
                }
                
                // Time pattern match
                if !session_patterns.session_time_distribution.is_empty() {
                    let hour = Utc::now().format("%H").to_string();
                    let hour_frequency = session_patterns.session_time_distribution
                        .get(&hour)
                        .cloned()
                        .unwrap_or(0.0);
                    
                    match_scores.push(hour_frequency);
                }
                
                // Day pattern match
                if !session_patterns.typical_days.is_empty() {
                    let day = Utc::now().format("%A").to_string();
                    let day_frequency = session_patterns.typical_days
                        .get(&day)
                        .cloned()
                        .unwrap_or(0.0);
                    
                    match_scores.push(day_frequency);
                }
                
                // Device match
                let device_match = profile.device_fingerprints.iter()
                    .find(|d| d.id == session_data.device_id);
                
                let device_score = if let Some(_) = device_match {
                    // Known device, use its usage frequency
                    let device_frequency = session_patterns.device_usage_ratios
                        .get(&session_data.device_id)
                        .cloned()
                        .unwrap_or(0.0);
                    
                    device_frequency
                } else {
                    // Unknown device has low match score
                    0.05
                };
                
                match_scores.push(device_score);
                
                // Session flow match
                if !session_patterns.typical_session_flow.is_empty() && !session_data.session_actions.is_empty() {
                    // Create flow pattern from sequential page visits
                    let flow = session_data.session_actions.iter()
                        .map(|a| a.page.clone())
                        .collect::<Vec<String>>()
                        .join(" > ");
                    
                    // Check if flow matches any typical flows
                    let flow_match = session_patterns.typical_session_flow.iter()
                        .any(|f| f.contains(&flow) || flow.contains(f));
                    
                    let flow_score = if flow_match {
                        0.8 // High match for typical flow
                    } else {
                        0.3 // Low match for atypical flow
                    };
                    
                    match_scores.push(flow_score);
                }
            }
        }
        
        // Calculate overall match score
        if match_scores.is_empty() {
            Ok(0.0) // No match if no comparison possible
        } else {
            let overall_match = match_scores.iter().sum::<f64>() / match_scores.len() as f64;
            Ok(overall_match)
        }
    }
    
    // Calculate device match score
    fn calculate_device_match_score(
        &self,
        profile: &UserBehaviorProfile,
        observations: &[&BehaviorObservation],
    ) -> Result<f64, String> {
        let mut match_scores = Vec::new();
        
        for obs in observations {
            if let Some(device_data) = &obs.device_fingerprint {
                // Check for matching device in profile
                for known_device in &profile.device_fingerprints {
                    let canvas_match = known_device.canvas_fingerprint == device_data.canvas_fingerprint;
                    let webgl_match = known_device.webgl_fingerprint == device_data.webgl_fingerprint;
                    let user_agent_match = known_device.user_agent == device_data.user_agent;
                    
                    // Calculate overall device match score
                    if canvas_match && webgl_match {
                        // Strong fingerprint match
                        match_scores.push(0.95);
                    } else if canvas_match || webgl_match {
                        // Partial fingerprint match
                        match_scores.push(0.7);
                    } else if user_agent_match {
                        // Only user agent matches
                        match_scores.push(0.4);
                    }
                }
                
                // If no matches found, device is unknown
                if match_scores.is_empty() {
                    match_scores.push(0.1); // Low match for unknown device
                }
            }
        }
        
        // Calculate overall match score
        if match_scores.is_empty() {
            Ok(0.0) // No match if no comparison possible
        } else {
            let overall_match = match_scores.iter().sum::<f64>() / match_scores.len() as f64;
            Ok(overall_match)
        }
    }
    
    // Get user profile
    pub async fn get_user_profile(&self, user_id: &str) -> Result<UserBehaviorProfile, String> {
        let profile_id = {
            let profile_by_user = self.profile_by_user.read().unwrap();
            profile_by_user.get(user_id)
                .cloned()
                .ok_or_else(|| "User profile not found".to_string())?
        };
        
        let profiles = self.profiles.read().unwrap();
        
        profiles.get(&profile_id)
            .cloned()
            .ok_or_else(|| "Profile not found".to_string())
    }
    
    // Lock a user profile
    pub async fn lock_profile(&self, user_id: &str, reason: &str) -> Result<(), String> {
        let profile_id = {
            let profile_by_user = self.profile_by_user.read().unwrap();
            profile_by_user.get(user_id)
                .cloned()
                .ok_or_else(|| "User profile not found".to_string())?
        };
        
        let mut profiles = self.profiles.write().unwrap();
        
        let profile = profiles.get_mut(&profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        profile.status = ProfileStatus::Locked;
        profile.updated_at = Utc::now();
        
        // Log security event
        self.security_monitoring.log_event(
            "ProfileLocked",
            Some(user_id),
            None,
            Some(&profile_id),
            "High",
            serde_json::json!({
                "reason": reason,
            }),
        ).await;
        
        Ok(())
    }
    
    // Unlock a user profile
    pub async fn unlock_profile(&self, user_id: &str, admin_id: &str) -> Result<(), String> {
        let profile_id = {
            let profile_by_user = self.profile_by_user.read().unwrap();
            profile_by_user.get(user_id)
                .cloned()
                .ok_or_else(|| "User profile not found".to_string())?
        };
        
        let mut profiles = self.profiles.write().unwrap();
        
        let profile = profiles.get_mut(&profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        if profile.status != ProfileStatus::Locked && profile.status != ProfileStatus::Suspended {
            return Err("Profile is not locked or suspended".to_string());
        }
        
        profile.status = ProfileStatus::Active;
        profile.updated_at = Utc::now();
        
        // Log security event
        self.security_monitoring.log_event(
            "ProfileUnlocked",
            Some(admin_id),
            None,
            Some(&profile_id),
            "Medium",
            serde_json::json!({
                "user_id": user_id,
            }),
        ).await;
        
        Ok(())
    }
    
    // Reset a user profile (back to learning phase)
    pub async fn reset_profile(&self, user_id: &str, admin_id: &str) -> Result<(), String> {
        let profile_id = {
            let profile_by_user = self.profile_by_user.read().unwrap();
            profile_by_user.get(user_id)
                .cloned()
                .ok_or_else(|| "User profile not found".to_string())?
        };
        
        let mut profiles = self.profiles.write().unwrap();
        
        let profile = profiles.get_mut(&profile_id)
            .ok_or_else(|| "Profile not found".to_string())?;
        
        // Keep device fingerprints but reset everything else
        let device_fingerprints = profile.device_fingerprints.clone();
        
        let now = Utc::now();
        
        *profile = UserBehaviorProfile {
            id: profile_id.clone(),
            user_id: user_id.to_string(),
            created_at: profile.created_at, // Keep original creation time
            updated_at: now,
            device_fingerprints, // Keep device fingerprints
            typing_patterns: None,
            mouse_patterns: None,
            session_patterns: SessionPatterns {
                typical_session_duration: 0.0,
                session_time_distribution: HashMap::new(),
                typical_days: HashMap::new(),
                typical_locations: Vec::new(),
                typical_ip_ranges: Vec::new(),
                device_usage_ratios: HashMap::new(),
                average_actions_per_session: 0.0,
                typical_session_flow: Vec::new(),
                last_updated: now,
            },
            transaction_patterns: TransactionPatterns {
                typical_transaction_amounts: Vec::new(),
                typical_transaction_times: HashMap::new(),
                typical_assets: HashMap::new(),
                typical_trading_pairs: HashMap::new(),
                typical_transaction_frequency: 0.0,
                typical_withdrawal_frequency: 0.0,
                typical_withdrawal_addresses: HashMap::new(),
                typical_deposit_sources: HashMap::new(),
                risk_appetite: 0.5,
                last_updated: now,
            },
            confidence_score: 0.0,
            anomaly_score: 0.0,
            status: ProfileStatus::Learning,
        };
        
        // Log security event
        self.security_monitoring.log_event(
            "ProfileReset",
            Some(admin_id),
            None,
            Some(&profile_id),
            "Medium",
            serde_json::json!({
                "user_id": user_id,
            }),
        ).await;
        
        Ok(())
    }
    
    // Update authentication thresholds
    pub async fn update_thresholds(
        &self,
        new_thresholds: AuthenticationThresholds,
    ) -> Result<(), String> {
        let mut thresholds = self.thresholds.write().unwrap();
        *thresholds = new_thresholds;
        Ok(())
    }
    
    // Get current authentication thresholds
    pub async fn get_thresholds(&self) -> AuthenticationThresholds {
        self.thresholds.read().unwrap().clone()
    }
    
    // Get recent observations for a user
    pub async fn get_user_observations(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Vec<BehaviorObservation> {
        let observations = self.observations.read().unwrap();
        
        observations.iter()
            .filter(|o| o.user_id == user_id)
            .rev() // Most recent first
            .take(limit)
            .cloned()
            .collect()
    }
    
    // Get anomalous observations for a user
    pub async fn get_anomalous_observations(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Vec<BehaviorObservation> {
        let observations = self.observations.read().unwrap();
        
        observations.iter()
            .filter(|o| o.user_id == user_id && o.is_anomaly.unwrap_or(false))
            .rev() // Most recent first
            .take(limit)
            .cloned()
            .collect()
    }
}

// Mock implementation of SecurityMonitoringInterface for testing
pub struct MockSecurityMonitoring;

#[async_trait::async_trait]
impl SecurityMonitoringInterface for MockSecurityMonitoring {
    async fn log_event(
        &self,
        _event_type: &str,
        _user_id: Option<&str>,
        _ip_address: Option<&str>,
        _resource_id: Option<&str>,
        _severity: &str,
        _details: serde_json::Value,
    ) -> String {
        // In a real implementation, this would log to a security monitoring system
        // For testing, just return a random ID
        Uuid::new_v4().to_string()
    }
}

// Example of how to use the behavioral biometrics service
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the security monitoring service
    let security_monitoring = Arc::new(MockSecurityMonitoring);
    
    // Create the behavioral biometrics service
    let biometrics_service = BehavioralBiometricsService::new(
        security_monitoring,
        10000, // Max 10,000 observations
    );
    
    // Create a user profile
    let user_id = "user123";
    let profile_id = biometrics_service.create_profile(user_id).await?;
    println!("Created profile {} for user {}", profile_id, user_id);
    
    // Record some observations
    
    // Device fingerprint observation
    let device_fingerprint = DeviceFingerprint {
        id: Uuid::new_v4().to_string(),
        name: Some("Work Laptop".to_string()),
        user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string(),
        browser: "Chrome".to_string(),
        browser_version: "91.0.4472.124".to_string(),
        operating_system: "macOS".to_string(),
        screen_resolution: "1920x1080".to_string(),
        color_depth: 24,
        timezone: "America/New_York".to_string(),
        language: "en-US".to_string(),
        plugins: vec!["PDF Viewer".to_string(), "Chrome PDF Viewer".to_string()],
        canvas_fingerprint: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce".to_string(),
        webgl_fingerprint: "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f".to_string(),
        fonts: vec!["Arial".to_string(), "Helvetica".to_string(), "Times New Roman".to_string()],
        ip_addresses: vec!["192.168.1.1".to_string()],
        first_seen: Utc::now(),
        last_seen: Utc::now(),
        is_trusted: true,
        risk_score: 0.1,
    };
    
    let device_observation = BehaviorObservation {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        session_id: "session123".to_string(),
        timestamp: Utc::now(),
        observation_type: ObservationType::DeviceFingerprint,
        device_fingerprint: Some(device_fingerprint),
        typing_data: None,
        mouse_data: None,
        session_data: None,
        transaction_data: None,
        anomaly_score: None,
        is_anomaly: None,
    };
    
    biometrics_service.record_observation(device_observation).await?;
    println!("Recorded device fingerprint observation");
    
    // Typing behavior observation
    let keystroke_times = vec![
        KeystrokeTime {
            key: "H".to_string(),
            key_down_time: 1000,
            key_up_time: 1060,
            key_code: 72,
        },
        KeystrokeTime {
            key: "e".to_string(),
            key_down_time: 1100,
            key_up_time: 1150,
            key_code: 69,
        },
        KeystrokeTime {
            key: "l".to_string(),
            key_down_time: 1200,
            key_up_time: 1250,
            key_code: 76,
        },
        KeystrokeTime {
            key: "l".to_string(),
            key_down_time: 1300,
            key_up_time: 1350,
            key_code: 76,
        },
        KeystrokeTime {
            key: "o".to_string(),
            key_down_time: 1400,
            key_up_time: 1460,
            key_code: 79,
        },
    ];
    
    let typing_data = TypingData {
        keystroke_times,
        context: "login_username".to_string(),
        typing_duration: 460,
        error_count: 0,
        backspace_count: 0,
    };
    
    let typing_observation = BehaviorObservation {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        session_id: "session123".to_string(),
        timestamp: Utc::now(),
        observation_type: ObservationType::TypingBehavior,
        device_fingerprint: None,
        typing_data: Some(typing_data),
        mouse_data: None,
        session_data: None,
        transaction_data: None,
        anomaly_score: None,
        is_anomaly: None,
    };
    
    biometrics_service.record_observation(typing_observation).await?;
    println!("Recorded typing behavior observation");
    
    // Get user profile
    let profile = biometrics_service.get_user_profile(user_id).await?;
    println!("User profile: {:?}", profile);
    
    // Authenticate user
    let auth_result = biometrics_service.authenticate_user(
        user_id,
        "session123",
        &[typing_observation.clone(), device_observation.clone()],
    ).await?;
    
    println!("Authentication result: {:?}", auth_result);
    
    Ok(())
}
// WorldClass Crypto Exchange: Behavioral Biometrics Implementation
// This file contains the behavioral biometrics security system

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::sync::Mutex;

///////////////////////////////////////////////////////////////////////////////
// Behavioral Biometrics Implementation
///////////////////////////////////////////////////////////////////////////////

// User behavior profile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserBehaviorProfile {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub device_fingerprints: Vec<DeviceFingerprint>,
    pub typing_patterns: Option<TypingPatterns>,
    pub mouse_patterns: Option<MousePatterns>,
    pub session_patterns: SessionPatterns,
    pub transaction_patterns: TransactionPatterns,
    pub confidence_score: f64, // 0.0 to 1.0
    pub anomaly_score: f64,    // 0.0 to 1.0
    pub status: ProfileStatus,
}

// Profile status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileStatus {
    Learning,
    Active,
    Suspended,
    Locked,
}

// Device fingerprint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub id: String,
    pub name: Option<String>,
    pub user_agent: String,
    pub operating_system: String,
    pub
