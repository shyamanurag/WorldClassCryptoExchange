use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use log::{info, debug, error};
use std::thread;

/// Metric types that can be collected
#[derive(Debug, Clone, PartialEq)]
pub enum MetricType {
    /// A counter that only increases
    Counter,
    /// A gauge that can go up or down
    Gauge,
    /// A histogram for distribution of values
    Histogram,
    /// A summary for distribution with quantiles
    Summary,
}

/// A collected metric
#[derive(Debug, Clone)]
pub struct Metric {
    /// Name of the metric
    pub name: String,
    /// Type of the metric
    pub metric_type: MetricType,
    /// Value of the metric
    pub value: f64,
    /// Labels associated with the metric
    pub labels: HashMap<String, String>,
    /// Timestamp of the metric
    pub timestamp: u64,
}

/// Metrics collector for the application
pub struct MetricsCollector {
    /// Prefix for all metrics
    prefix: String,
    /// Collected counters
    counters: RwLock<HashMap<String, f64>>,
    /// Collected gauges
    gauges: RwLock<HashMap<String, f64>>,
    /// Collected histograms
    histograms: RwLock<HashMap<String, Vec<f64>>>,
    /// Collected metrics with labels
    metrics: Mutex<Vec<Metric>>,
    /// Start time of the collector
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(prefix: &str) -> Self {
        MetricsCollector {
            prefix: prefix.to_string(),
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            metrics: Mutex::new(Vec::new()),
            start_time: Instant::now(),
        }
    }
    
    /// Get the prefix
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    
    /// Get the uptime in seconds
    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// Increment a counter by a value
    pub fn increment_counter(&self, name: &str, value: f64) {
        let key = format!("{}.{}", self.prefix, name);
        let mut counters = self.counters.write().unwrap();
        *counters.entry(key.clone()).or_insert(0.0) += value;
        
        // Add to metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(Metric {
            name: key,
            metric_type: MetricType::Counter,
            value,
            labels: HashMap::new(),
            timestamp: self.current_timestamp(),
        });
    }
    
    /// Increment a counter with labels
    pub fn increment_counter_with_labels(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let key = format!("{}.{}", self.prefix, name);
        let mut counters = self.counters.write().unwrap();
        
        // Generate a key that includes the labels
        let mut label_key = key.clone();
        for (k, v) in &labels {
            label_key = format!("{}:{}{}", label_key, k, v);
        }
        
        *counters.entry(label_key).or_insert(0.0) += value;
        
        // Add to metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(Metric {
            name: key,
            metric_type: MetricType::Counter,
            value,
            labels,
            timestamp: self.current_timestamp(),
        });
    }
    
    /// Set a gauge to a specific value
    pub fn set_gauge(&self, name: &str, value: f64) {
        let key = format!("{}.{}", self.prefix, name);
        let mut gauges = self.gauges.write().unwrap();
        gauges.insert(key.clone(), value);
        
        // Add to metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(Metric {
            name: key,
            metric_type: MetricType::Gauge,
            value,
            labels: HashMap::new(),
            timestamp: self.current_timestamp(),
        });
    }
    
    /// Set a gauge with labels
    pub fn set_gauge_with_labels(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let key = format!("{}.{}", self.prefix, name);
        let mut gauges = self.gauges.write().unwrap();
        
        // Generate a key that includes the labels
        let mut label_key = key.clone();
        for (k, v) in &labels {
            label_key = format!("{}:{}{}", label_key, k, v);
        }
        
        gauges.insert(label_key, value);
        
        // Add to metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(Metric {
            name: key,
            metric_type: MetricType::Gauge,
            value,
            labels,
            timestamp: self.current_timestamp(),
        });
    }
    
    /// Record a value in a histogram
    pub fn observe_histogram(&self, name: &str, value: f64) {
        let key = format!("{}.{}", self.prefix, name);
        let mut histograms = self.histograms.write().unwrap();
        histograms.entry(key.clone()).or_insert_with(Vec::new).push(value);
        
        // Add to metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(Metric {
            name: key,
            metric_type: MetricType::Histogram,
            value,
            labels: HashMap::new(),
            timestamp: self.current_timestamp(),
        });
    }
    
    /// Record a value in a histogram with labels
    pub fn observe_histogram_with_labels(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let key = format!("{}.{}", self.prefix, name);
        let mut histograms = self.histograms.write().unwrap();
        
        // Generate a key that includes the labels
        let mut label_key = key.clone();
        for (k, v) in &labels {
            label_key = format!("{}:{}{}", label_key, k, v);
        }
        
        histograms.entry(label_key).or_insert_with(Vec::new).push(value);
        
        // Add to metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(Metric {
            name: key,
            metric_type: MetricType::Histogram,
            value,
            labels,
            timestamp: self.current_timestamp(),
        });
    }
    
    /// Record order processing time
    pub fn record_order_processing_time(&self, symbol: &str, time_us: u64) {
        let mut labels = HashMap::new();
        labels.insert("symbol".to_string(), symbol.to_string());
        
        self.observe_histogram_with_labels(
            "order_processing_time_us",
            time_us as f64,
            labels,
        );
    }
    
    /// Record order book operation time
    pub fn record_order_book_operation_time(&self, operation: &str, symbol: &str, time_us: u64) {
        let mut labels = HashMap::new();
        labels.insert("operation".to_string(), operation.to_string());
        labels.insert("symbol".to_string(), symbol.to_string());
        
        self.observe_histogram_with_labels(
            "order_book_operation_time_us",
            time_us as f64,
            labels,
        );
    }
    
    /// Record API request time
    pub fn record_api_request_time(&self, endpoint: &str, method: &str, time_ms: u64) {
        let mut labels = HashMap::new();
        labels.insert("endpoint".to_string(), endpoint.to_string());
        labels.insert("method".to_string(), method.to_string());
        
        self.observe_histogram_with_labels(
            "api_request_time_ms",
            time_ms as f64,
            labels,
        );
    }
    
    /// Record database query time
    pub fn record_db_query_time(&self, operation: &str, table: &str, time_ms: u64) {
        let mut labels = HashMap::new();
        labels.insert("operation".to_string(), operation.to_string());
        labels.insert("table".to_string(), table.to_string());
        
        self.observe_histogram_with_labels(
            "db_query_time_ms",
            time_ms as f64,
            labels,
        );
    }
    
    /// Get the current timestamp
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
    }
    
    /// Log all metrics
    pub fn log_metrics(&self) {
        debug!("Logging metrics...");
        
        // Log counters
        let counters = self.counters.read().unwrap();
        for (key, value) in counters.iter() {
            debug!("METRIC [Counter] {} = {}", key, value);
        }
        
        // Log gauges
        let gauges = self.gauges.read().unwrap();
        for (key, value) in gauges.iter() {
            debug!("METRIC [Gauge] {} = {}", key, value);
        }
        
        // Log histograms
        let histograms = self.histograms.read().unwrap();
        for (key, values) in histograms.iter() {
            if !values.is_empty() {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                
                let len = sorted.len();
                let min = sorted[0];
                let max = sorted[len - 1];
                let p50_idx = (len as f64 * 0.5) as usize;
                let p90_idx = (len as f64 * 0.9) as usize;
                let p95_idx = (len as f64 * 0.95) as usize;
                let p99_idx = (len as f64 * 0.99) as usize;
                
                debug!(
                    "METRIC [Histogram] {} = min:{:.2} max:{:.2} p50:{:.2} p90:{:.2} p95:{:.2} p99:{:.2} count:{}",
                    key, min, max, 
                    sorted[p50_idx.min(len - 1)],
                    sorted[p90_idx.min(len - 1)],
                    sorted[p95_idx.min(len - 1)],
                    sorted[p99_idx.min(len - 1)],
                    len
                );
            }
        }
    }
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        // Export counters
        let counters = self.counters.read().unwrap();
        for (key, value) in counters.iter() {
            output.push_str(&format!("{} {}\n", key, value));
        }
        
        // Export gauges
        let gauges = self.gauges.read().unwrap();
        for (key, value) in gauges.iter() {
            output.push_str(&format!("{} {}\n", key, value));
        }
        
        // Export histograms
        let histograms = self.histograms.read().unwrap();
        for (key, values) in histograms.iter() {
            if !values.is_empty() {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                
                let len = sorted.len();
                let min = sorted[0];
                let max = sorted[len - 1];
                let sum: f64 = values.iter().sum();
                
                output.push_str(&format!("{}_count {}\n", key, len));
                output.push_str(&format!("{}_sum {}\n", key, sum));
                output.push_str(&format!("{}_min {}\n", key, min));
                output.push_str(&format!("{}_max {}\n", key, max));
                
                // Add percentiles
                let p50_idx = (len as f64 * 0.5) as usize;
                let p90_idx = (len as f64 * 0.9) as usize;
                let p95_idx = (len as f64 * 0.95) as usize;
                let p99_idx = (len as f64 * 0.99) as usize;
                
                output.push_str(&format!("{{{}_quantile=\"0.5\"}} {}\n", key, sorted[p50_idx.min(len - 1)]));
                output.push_str(&format!("{{{}_quantile=\"0.9\"}} {}\n", key, sorted[p90_idx.min(len - 1)]));
                output.push_str(&format!("{{{}_quantile=\"0.95\"}} {}\n", key, sorted[p95_idx.min(len - 1)]));
                output.push_str(&format!("{{{}_quantile=\"0.99\"}} {}\n", key, sorted[p99_idx.min(len - 1)]));
            }
        }
        
        output
    }
}

/// Start a background thread to regularly log metrics
pub fn start_metrics_logging(metrics: Arc<MetricsCollector>, interval_secs: u64) {
    thread::spawn(move || {
        info!("Starting metrics logging thread with interval {}s", interval_secs);
        
        loop {
            thread::sleep(Duration::from_secs(interval_secs));
            metrics.log_metrics();
        }
    });
}

/// Initializes metrics collection for the application
pub fn init_metrics(config: &crate::config::Config) -> Result<Arc<MetricsCollector>, String> {
    let metrics = Arc::new(MetricsCollector::new(&config.metrics_prefix));
    
    // Set initial gauges
    metrics.set_gauge("process_start_time", 
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as f64
    );
    
    // Start metrics logging thread
    start_metrics_logging(Arc::clone(&metrics), 60);
    
    info!("Metrics initialized with prefix: {}", config.metrics_prefix);
    
    Ok(metrics)
}
