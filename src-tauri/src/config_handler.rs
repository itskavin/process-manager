use crate::types::ProcessConfig;
use std::fs;
use std::path::PathBuf;

pub struct ConfigHandler;

impl ConfigHandler {
    pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA")?;
            let path = PathBuf::from(app_data).join("ProcessManager");
            fs::create_dir_all(&path)?;
            Ok(path)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME")?;
            let path = PathBuf::from(home).join(".process-manager");
            fs::create_dir_all(&path)?;
            Ok(path)
        }
    }

    pub fn get_logs_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = Self::get_config_dir()?;
        let logs_dir = config_dir.join("logs");
        fs::create_dir_all(&logs_dir)?;
        Ok(logs_dir)
    }

    pub fn load_configs() -> Result<Vec<ProcessConfig>, Box<dyn std::error::Error>> {
        let config_dir = Self::get_config_dir()?;
        let config_file = config_dir.join("processes.json");

        if !config_file.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&config_file)?;
        let configs: Vec<ProcessConfig> = serde_json::from_str(&contents)?;
        Ok(configs)
    }

    pub fn save_configs(configs: &[ProcessConfig]) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = Self::get_config_dir()?;
        let config_file = config_dir.join("processes.json");

        let contents = serde_json::to_string_pretty(configs)?;
        fs::write(&config_file, contents)?;
        Ok(())
    }
}
