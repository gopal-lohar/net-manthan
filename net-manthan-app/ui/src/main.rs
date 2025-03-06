use async_std::task::sleep;
use dioxus::prelude::*;
use download_engine::{
    config::NetManthanConfig,
    types::{IpcRequest, IpcResponse},
    Download,
};
use std::path::PathBuf;
use std::time::Duration;
use utils::Client;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Create state to store downloads
    let mut downloads = use_signal(Vec::<Download>::new);

    // Set up config locally
    let config =
        use_memo(
            || match NetManthanConfig::load_config(PathBuf::from("./.dev/config.toml")) {
                Ok(config) => config,
                Err(_) => NetManthanConfig::get_default_config(),
            },
        );

    let mut client = use_signal(|| Client::new(&config.read().get_ipc_server_address()).ok());

    // Function to fetch downloads
    let mut fetch_downloads = move || {
        if let Some(client) = &mut *client.write() {
            match client.send_and_receive(IpcRequest::GetActiveDownloads {}) {
                Ok(IpcResponse::DownloadsList(list)) => {
                    downloads.set(list);
                }
                Ok(_) => {
                    println!("Received unexpected response type");
                }
                Err(err) => {
                    println!("Failed to fetch downloads: {}", err);
                }
            }
        }
    };

    let mut should_fetch = use_signal(|| true);
    use_effect(move || {
        if !*should_fetch.read() {
            return;
        }

        // Reset the flag
        should_fetch.set(false);

        // Fetch downloads
        fetch_downloads();

        // Schedule the next fetch
        let mut should_fetch_clone = should_fetch.clone();
        spawn(async move {
            sleep(Duration::from_millis(500)).await;
            should_fetch_clone.set(true);
        });
    });

    // Initial fetch on component mount
    use_effect(move || {
        fetch_downloads();
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Container {
            downloads: downloads.read().clone()
        }
    }
}

#[component]
pub fn Container(downloads: Vec<Download>) -> Element {
    rsx! {
        div { class: "container",
            h1 { "Download Manager" }
            DownloadList { downloads: downloads }
        }
    }
}

#[component]
pub fn DownloadList(downloads: Vec<Download>) -> Element {
    rsx! {
        div { class: "download-list",
            if downloads.is_empty() {
                div { class: "no-downloads", "No active downloads" }
            } else {
                {downloads.iter().map(|download| {
                    rsx! {
                        DownloadItem { key: "{download.download_id}", download: download.clone() }
                    }
                })}
            }
        }
    }
}

#[component]
pub fn DownloadItem(download: Download) -> Element {
    rsx! {
        div { class: "download-item",
            div { class: "download-name", "{download.filename}" }
        }
    }
}
