use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetManthanConfig {
    pub auto_resume: bool,
    pub thread_count: u8,
    pub update_interval_in_ms: i64,
    pub single_threaded_buffer_size_in_kb: u64,
    pub multi_threaded_buffer_size_in_kb: u64,
    pub ipc_server_address: String,
    pub ipc_server_port: u16,
    pub download_dir: PathBuf,
    pub database_path: PathBuf,
    pub log_path: PathBuf,
}

// it's here just because that way it's easier to change things
impl NetManthanConfig {
    pub fn get_default_config() -> Self {
        NetManthanConfig {
            auto_resume: false,
            thread_count: 5,
            update_interval_in_ms: 1000,
            single_threaded_buffer_size_in_kb: 1024,
            multi_threaded_buffer_size_in_kb: 1024,
            ipc_server_address: String::from("127.0.0.1"),
            ipc_server_port: 8814,
            download_dir: PathBuf::from("./.dev/downloads"),
            database_path: PathBuf::from("./.dev/downloads.db"),
            log_path: PathBuf::from("./.dev/logs/"),
        }
    }
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
}
