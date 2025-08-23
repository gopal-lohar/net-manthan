use std::{process::Command, sync::Arc};
use tracing::{info, warn};
use utils::rpc::{NativeRpcSettings, RpcConfig, client::Client};

pub struct DaemonManager {
    config: RpcConfig,
}

impl DaemonManager {
    pub fn new() -> Self {
        Self {
            config: RpcConfig {
                native_rpc_settings: NativeRpcSettings {
                    address: "/tmp/vayu.sock".into(),
                    allow_all_users: true,
                },
            },
        }
    }

    pub async fn get_client_handle(&self) -> Arc<Client> {
        let client = self.connect_to_daemon().await;
        if client.is_connected().await {
            client
        } else {
            self.start_daemon_with_fallback().await;
            self.connect_to_daemon().await
        }
    }

    async fn connect_to_daemon(&self) -> Arc<Client> {
        let client = Client::new(
            "".into(),
            RpcConfig {
                native_rpc_settings: NativeRpcSettings {
                    address: "/tmp/vayu.sock".into(),
                    allow_all_users: true,
                },
            },
        );
        let _ = client.connect().await;
        Arc::new(client)
    }

    async fn start_daemon_with_fallback(&self) {
        #[cfg(target_os = "linux")]
        {
            if self.try_systemd_start().await.is_ok() {
                return;
            }
        }
        let _ = self.start_daemon_manually().await;
    }

    #[cfg(target_os = "linux")]
    async fn try_systemd_start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if service exists and is enabled

        use std::process::Command;

        use tracing::warn;
        let status = Command::new("systemctl")
            .args(["--user", "is-enabled", "vayuget.service"])
            .output()?;

        if status.status.success() {
            info!("Starting daemon via systemd...");
            let start = Command::new("systemctl")
                .args(["--user", "start", "vayuget.service"])
                .output()?;

            if start.status.success() {
                return Ok(());
            }
        }

        warn!("Systemd service not available or failed to start");
        Err("Systemd service not available or failed to start".into())
    }

    async fn start_daemon_manually(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting daemon manually...");

        #[cfg(unix)]
        {
            let _ = std::fs::remove_file(&self.config.native_rpc_settings.address);
        }
        #[cfg(debug_assertions)]
        let mut cmd = Command::new("./target/debug/vayuget");
        #[cfg(not(debug_assertions))]
        let mut cmd = Command::new("vayuget");

        cmd.args(&["--daemon", "--log-level=trace"]);

        // #[cfg(unix)]
        // {
        //     cmd.stdout(Stdio::null())
        //         .stderr(Stdio::null())
        //         .stdin(Stdio::null());
        // }

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x00000008); // CREATE_NO_WINDOW
        }

        let child = cmd.spawn()?;
        info!("{:?}", child);
        // Let the daemon detach itself
        Ok(())
    }
}
