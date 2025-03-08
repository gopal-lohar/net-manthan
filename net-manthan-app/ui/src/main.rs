mod components;

use async_std::task::sleep;
use components::{Dialog, DownloadStatus, Pages, Sidebar};
use dioxus::prelude::*;
use download_engine::{
    config::NetManthanConfig,
    types::{IpcRequest, IpcResponse},
    Download,
};
use std::path::PathBuf;
use std::time::Duration;
use utils::format_bytes;
use utils::Client;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const PREFLIGHT_CSS: Asset = asset!("/assets/preflight.css");
const UTILS_CSS: Asset = asset!("/assets/utils.css");
const DIALOG_CSS: Asset = asset!("/assets/dialog.css");
const SIDEBAR_CSS: Asset = asset!("/assets/sidebar.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut downloads = use_signal(Vec::<Download>::new);
    let current_page = use_signal(|| Pages::Downloads { status: None });

    // Set up config locally
    let config =
        use_memo(
            || match NetManthanConfig::load_config(PathBuf::from("./.dev/config.toml")) {
                Ok(config) => config,
                Err(_) => NetManthanConfig::get_default_config(),
            },
        );

    let mut client = use_signal(|| Client::new(&config().get_ipc_server_address()).ok());

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

    use_future(move || async move {
        loop {
            fetch_downloads();
            sleep(Duration::from_millis(500)).await;
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: PREFLIGHT_CSS }
        document::Link { rel: "stylesheet", href: UTILS_CSS }
        document::Link { rel: "stylesheet", href: DIALOG_CSS }
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        div { class: "app",
            Sidebar{
                current_page
            },
            match current_page() {
                Pages::Downloads { status } => {
                    rsx!{
                        MainContainer {
                            status,
                            client: client.clone(),
                            downloads: downloads().clone()
                        }
                    }
                }
                Pages::Settings => {
                    rsx!{
                        "Settings"
                    }
                }
                Pages::About => {
                    rsx!{
                        "About"
                    }
                }
            }
        }
    }
}

#[component]
pub fn MainContainer(
    status: Option<DownloadStatus>,
    downloads: Vec<Download>,
    client: Signal<Option<Client>>,
) -> Element {
    let show_dialog = use_signal(|| false);
    rsx! {
        div { class: "main-container",
            div { class: "container",
                TopBar {
                    status,
                    show_dialog: show_dialog.clone()
                }
                DownloadList { downloads: downloads }
                if show_dialog() == true {
                    Dialog {client: client.clone(), show_dialog}
                }
            }
        }
    }
}

#[component]
pub fn TopBar(status: Option<DownloadStatus>, show_dialog: Signal<bool>) -> Element {
    rsx! {
        div { class: "top-bar flex items-center justify-between",
            h1 {match status {
                Some(status) => match status {
                    DownloadStatus::Downloading => "Active",
                    DownloadStatus::Queued => "Queued",
                    DownloadStatus::Finished => "Finished",
                    DownloadStatus::Failed => "Failed",
                    DownloadStatus::Cancelled => "Cancelled",
                    DownloadStatus::Paused => "Paused",
                },
                None => "All Downloads",
            }},
            button {class: "download-dialog-button flex items-center justify-center", onclick: move |_| *show_dialog.write() = true, "" }
        }
    }
}

#[component]
pub fn DownloadList(downloads: Vec<Download>) -> Element {
    rsx! {
        div { class: "download-list",
            if downloads.is_empty() {
                div { class: "no-downloads flex items-center justify-center", "No active downloads" }
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
    let total: u64 = download.parts.iter().map(|p| p.total_bytes).sum();
    let downloaded: u64 = download.parts.iter().map(|p| p.bytes_downloaded).sum();
    let progress = if total > 0 {
        (downloaded as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    rsx! {
        div { class: "download-item flex flex-column",
            div { class: "download-name", "{download.filename}" }
            div { class: "progress-bar", "data-progress": "70%", style: "--progress: {progress}%"}
            div { class: "flex items-center justify-between",
                div{"{format_bytes(downloaded)}/{format_bytes(total)}"},
                div{"{progress:.2}%"}
            }
        }
    }
}
