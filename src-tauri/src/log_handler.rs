use crate::types::LogEntry;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct LogHandler {
    log_dir: PathBuf,
}

impl LogHandler {
    pub fn new(log_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(&log_dir)?;
        Ok(LogHandler { log_dir })
    }

    pub fn get_log_file(&self, process_id: &str) -> PathBuf {
        self.log_dir.join(format!("{}.log", process_id))
    }

    pub fn write_log(&self, process_id: &str, level: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let log_path = self.get_log_file(process_id);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);
        file.write_all(log_line.as_bytes())?;

        Ok(())
    }

    pub fn read_logs(&self, process_id: &str, limit: Option<usize>) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let log_path = self.get_log_file(process_id);
        
        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&log_path)?;
        let mut logs: Vec<LogEntry> = content
            .lines()
            .filter_map(|line| {
                if line.is_empty() {
                    return None;
                }
                
                // Parse format: [timestamp] [level] message
                if let Some(timestamp_end) = line.find(']') {
                    if let Some(level_end) = line[timestamp_end + 1..].find(']') {
                        let timestamp = line[1..timestamp_end].to_string();
                        let level = line[timestamp_end + 2..timestamp_end + 2 + level_end].trim().to_string();
                        let message = line[timestamp_end + 3 + level_end..].trim().to_string();
                        
                        return Some(LogEntry {
                            timestamp,
                            level,
                            message,
                        });
                    }
                }
                None
            })
            .collect();

        // Apply limit (last N entries)
        if let Some(limit_val) = limit {
            if logs.len() > limit_val {
                let start = logs.len() - limit_val;
                logs = logs[start..].to_vec();
            }
        }

        Ok(logs)
    }

    pub fn clear_logs(&self, process_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let log_path = self.get_log_file(process_id);
        if log_path.exists() {
            fs::remove_file(log_path)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn rotate_log(&self, process_id: &str, max_size_mb: u64) -> Result<(), Box<dyn std::error::Error>> {
        let log_path = self.get_log_file(process_id);
        
        if log_path.exists() {
            let metadata = std::fs::metadata(&log_path)?;
            let size_mb = metadata.len() / (1024 * 1024);
            
            if size_mb > max_size_mb {
                let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                let backup_name = format!("{}.{}.log", process_id, timestamp);
                let backup_path = self.log_dir.join(backup_name);
                fs::rename(&log_path, &backup_path)?;
            }
        }

        Ok(())
    }
}
