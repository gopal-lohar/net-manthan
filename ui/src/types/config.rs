use super::theme::ThemeOptions;
use iced::{Size, Task, window};
use utils::rpc::{NativeRpcSettings, RpcConfig};

#[derive(Clone)]
pub struct Config {
    pub ui: UiConfig,
    pub rpc: RpcConfig,
}

#[derive(Clone)]
pub struct UiConfig {
    pub custom_decoration: bool,
    pub theme: ThemeOptions,
    pub scale_factor: f64,
    pub size: Size,
    pub maximized: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: UiConfig {
                custom_decoration: true,
                theme: ThemeOptions::Moonfly,
                scale_factor: 1.,
                size: Size::new(1024.0, 768.0),
                maximized: false,
            },
            rpc: RpcConfig {
                native_rpc_settings: NativeRpcSettings {
                    address: "/tmp/vayu.sock".into(),
                    allow_all_users: true,
                },
            },
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
                self.size = size;
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
