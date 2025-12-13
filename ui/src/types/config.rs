use super::theme::ThemeOptions;
use iced::{Size, Task, window};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utils::config;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub ui: UiConfig,
    pub vayuget: config::Vayuget, // comes from the utils crate
    pub rpc: config::RpcConfig,   // comes from the utils crate
}

impl Default for Config {
    fn default() -> Self {
        let config = config::Config::default();
        Self {
            ui: UiConfig::default(),
            vayuget: config.vayuget,
            rpc: config.rpc,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub custom_decoration: bool,
    pub theme: ThemeOptions,
    pub scale_factor: f64,
    pub size: SerializableSize,
    pub maximized: bool,
    pub vayuget_path: PathBuf,
    pub config_path: PathBuf,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            custom_decoration: true,
            theme: ThemeOptions::Moonfly,
            scale_factor: 1.,
            size: SerializableSize::new(1024.0, 768.0),
            maximized: false,
            vayuget_path: PathBuf::from("/usr/bin/vayuget"), // updated from the daemon manager
            config_path: Self::get_config_path(),
        }
    }
}

impl UiConfig {
    pub fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .map(|p| p.join("net-manthan").join("ui.toml"))
            .unwrap()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableSize {
    pub width: f32,
    pub height: f32,
}

impl SerializableSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl From<SerializableSize> for Size {
    fn from(size: SerializableSize) -> Self {
        Size::new(size.width, size.height)
    }
}

impl From<Size> for SerializableSize {
    fn from(size: Size) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpdateUiConfig {
    CustomDecoration(bool),
    Theme(ThemeOptions),
    ScaleFactor(f64),
    Size(Size),
    Maximized(bool),
}

impl UiConfig {
    pub fn update(&mut self, message: UpdateUiConfig) -> Task<UpdateUiConfig> {
        match message {
            UpdateUiConfig::CustomDecoration(value) => {
                if value != self.custom_decoration {
                    self.custom_decoration = value;
                    window::get_latest().and_then(window::toggle_decorations)
                } else {
                    Task::none()
                }
            }
            UpdateUiConfig::Theme(value) => {
                self.theme = value;
                Task::none()
            }
            UpdateUiConfig::ScaleFactor(value) => {
                self.scale_factor = value;
                Task::none()
            }
            UpdateUiConfig::Size(size) => {
                self.size = size.into();
                window::get_latest()
                    .and_then(window::get_maximized)
                    .map(UpdateUiConfig::Maximized)
            }
            UpdateUiConfig::Maximized(maximized) => {
                self.maximized = maximized;
                Task::none()
            }
        }
    }
}
