#[cfg(unix)]
use tokio::{signal::unix::{SignalKind, signal}};
use tokio::sync::oneshot;
use tracing::info;

pub fn ctrl_c() -> oneshot::Receiver<()> {
    let (tx, rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        #[cfg(windows)]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl_c signal");
            info!("\nCtrl+C received. Sending shutdown signal...");
        }

        #[cfg(unix)]
        {
            let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler");
            let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
            tokio::select! {
                _ = sigint.recv() => {},
                _ = sigterm.recv() => {},
            }
            info!("SIGTERM received. Sending shutdown signal...");
        }

        #[cfg(any(windows, unix))]
        {
            let _ = tx.send(());
        }
    });

    rx
}
