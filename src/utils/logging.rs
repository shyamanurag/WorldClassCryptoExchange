use env_logger::{Builder, Env};
use log::{LevelFilter, info, debug, error};
use std::io::Write;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::sync::Once;
use chrono::{Utc, Local};

static INIT: Once = Once::new();

/// Initialize the logging system
pub fn init_logger() {
    INIT.call_once(|| {
        // Create log directory if it doesn't exist
        let log_dir = get_log_dir();
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory: {}", e);
        }

        // Get log file path
        let log_file = get_log_file_path(&log_dir);
        
        // Get log level from environment
        let env = Env::default()
            .filter_or("LOG_LEVEL", "info");

        // Create file logger
        match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_file)
        {
            Ok(file) => {
                let mut builder = Builder::from_env(env);
                builder
                    .format(|buf, record| {
                        writeln!(
                            buf,
                            "{} [{}] - {}: {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            record.level(),
                            record.target(),
                            record.args()
                        )
                    })
                    .filter(None, LevelFilter::Info)
                    .target(env_logger::Target::Pipe(Box::new(FileAndStdout {
                        file,
                    })))
                    .init();

                info!("Logging initialized: {}", log_file.display());
                debug!("Log level: {}", get_log_level());
                info!("WorldClass Crypto Exchange starting at {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
            }
            Err(e) => {
                eprintln!("Failed to open log file: {}", e);
                
                // Fall back to stdout only
                let mut builder = Builder::from_env(env);
                builder
                    .format(|buf, record| {
                        writeln!(
                            buf,
                            "{} [{}] - {}: {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            record.level(),
                            record.target(),
                            record.args()
                        )
                    })
                    .filter(None, LevelFilter::Info)
                    .init();

                error!("Failed to open log file, logging to stdout only: {}", e);
            }
        }

        // Clean up old log files
        if let Err(e) = clean_old_logs(&log_dir) {
            error!("Failed to clean old logs: {}", e);
        }
    });
}

/// Get the log directory path
fn get_log_dir() -> PathBuf {
    let mut path = if let Ok(dir) = env::var("LOG_DIR") {
        PathBuf::from(dir)
    } else {
        match home::home_dir() {
            Some(path) => path.join(".worldclass_crypto_exchange").join("logs"),
            None => {
                eprintln!("Could not determine home directory for logs");
                PathBuf::from("logs")
            }
        }
    };

    // Create intermediate directories
    if !path.exists() {
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Failed to create log directory: {}", e);
        }
    }

    path
}

/// Get the log file path for the current session
fn get_log_file_path(log_dir: &PathBuf) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    log_dir.join(format!("exchange_{}.log", timestamp))
}

/// Get the current log level
fn get_log_level() -> String {
    env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
}

/// Clean up old log files (keep only the last 10)
fn clean_old_logs(log_dir: &PathBuf) -> std::io::Result<()> {
    let mut log_files = Vec::new();
    
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(extension) = path.extension() {
            if extension == "log" && path.is_file() {
                log_files.push(path);
            }
        }
    }
    
    // Sort by modification time (newest first)
    log_files.sort_by(|a, b| {
        let a_meta = fs::metadata(a).unwrap();
        let b_meta = fs::metadata(b).unwrap();
        b_meta.modified().unwrap().cmp(&a_meta.modified().unwrap())
    });
    
    // Keep only the latest 10 log files
    const MAX_LOG_FILES: usize = 10;
    
    if log_files.len() > MAX_LOG_FILES {
        for file in log_files.iter().skip(MAX_LOG_FILES) {
            debug!("Removing old log file: {}", file.display());
            fs::remove_file(file)?;
        }
    }
    
    Ok(())
}

/// Custom writer that writes to both a file and stdout
struct FileAndStdout {
    file: File,
}

impl Write for FileAndStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to stdout
        std::io::stdout().write_all(buf)?;
        
        // Write to file
        self.file.write_all(buf)?;
        
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()?;
        self.file.flush()?;
        Ok(())
    }
}
