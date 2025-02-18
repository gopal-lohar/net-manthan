use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetManthanConfig {
    pub auto_resume: bool,
    pub default_threads: u8,
    pub single_threaded_buffer_size_in_kb: u64,
    pub multi_threaded_buffer_size_in_kb: u64,
    pub download_dir: PathBuf,
    pub database_path: PathBuf,
    pub log_path: PathBuf,
}

impl NetManthanConfig {
    pub fn load_config(config_path: PathBuf) -> Result<Self> {
        // Check if config file exists
        if !config_path.exists() {
            // Create default config
            let default_config = Self::get_default_config();
            default_config.save_config(config_path)?;
            return Ok(default_config);
        }

        let config_str = fs::read_to_string(config_path)?;

        // Parse TOML
        let config: NetManthanConfig = toml::from_str(&config_str)?;

        Ok(config)
    }

    pub fn save_config(&self, config_path: PathBuf) -> Result<()> {
        let toml_string = toml::to_string(self)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to file
        let mut file = fs::File::create(config_path)?;
        file.write_all(toml_string.as_bytes())?;

        Ok(())
    }

    pub fn get_default_config() -> Self {
        NetManthanConfig {
            auto_resume: false,
            default_threads: 5,
            single_threaded_buffer_size_in_kb: 1024,
            multi_threaded_buffer_size_in_kb: 1024,
            download_dir: PathBuf::from("./.dev/downloads"),
            database_path: PathBuf::from("./.dev/downloads.db"),
            log_path: PathBuf::from("./.dev/log.txt"),
        }
    }
}
