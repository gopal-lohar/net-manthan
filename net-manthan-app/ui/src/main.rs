mod components;

use chrono::Utc;
use components::{Dialog, Pages, Sidebar};
use dioxus::prelude::*;
use download_engine::{
    config::NetManthanConfig,
    types::{DownloadStatus, IpcRequest, IpcResponse},
    Download,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use utils::format_bytes;
use utils::Client;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const PREFLIGHT_CSS: Asset = asset!("/assets/preflight.css");
const UTILS_CSS: Asset = asset!("/assets/utils.css");
const DIALOG_CSS: Asset = asset!("/assets/dialog.css");
const SIDEBAR_CSS: Asset = asset!("/assets/sidebar.css");

const PAUSE_ICON: Asset = asset!("/assets/icons/pause_circle_icon.svg");
const PLAY_ICON: Asset = asset!("/assets/icons/play_circle_icon.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let show_dialog = use_signal(|| false);
    let mut downloads = use_signal(Vec::<Download>::new);
    let current_page = use_signal(|| Pages::Downloading);

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
            match client.send_and_receive(IpcRequest::GetDownloads({
                let mut status_vec = Vec::new();
                match current_page() {
                    Pages::Downloading => {
                        status_vec.push(DownloadStatus::Connecting);
                        status_vec.push(DownloadStatus::Downloading);
                    }
                    Pages::Queued => {
                        status_vec.push(DownloadStatus::Queued);
                    }
                    Pages::Stopped => {
                        status_vec.push(DownloadStatus::Paused);
                        status_vec.push(DownloadStatus::Failed(String::new()));
                        status_vec.push(DownloadStatus::Completed(Utc::now()));
                        status_vec.push(DownloadStatus::Cancelled);
                    }
                    _ => {}
                }
                status_vec
            })) {
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
        MainContainer {
            current_page,
            client: client.clone(),
            downloads: downloads().clone(),
            show_dialog
        }
        if show_dialog() == true {
            Dialog {client: client.clone(), show_dialog}
        }

        }
    }
}

#[component]
pub fn MainContainer(
    current_page: Signal<Pages>,
    downloads: Vec<Download>,
    client: Signal<Option<Client>>,
    show_dialog: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "main-container",
            div { class: "container",
                TopBar {
                    current_page,
                    show_dialog
                }
                DownloadList { client, downloads }
            }
        }
    }
}

#[component]
pub fn TopBar(current_page: Signal<Pages>, show_dialog: Signal<bool>) -> Element {
    rsx! {
        div { class: "top-bar flex items-center justify-between",
            h1 {match current_page() {
                Pages::Downloading => "Downloading",
                Pages::Queued => "Queued",
                Pages::Stopped => "Stopped",
                Pages::Settings => "Settings",
                Pages::About => "About"
            }},
            button {class: "download-dialog-button flex items-center justify-center", onclick: move |_| *show_dialog.write() = true, "" }
        }
    }
}

#[component]
pub fn DownloadList(client: Signal<Option<Client>>, downloads: Vec<Download>) -> Element {
    rsx! {
        div { class: "download-list",
            if downloads.is_empty() {
                div { class: "no-downloads flex items-center justify-center", "No active downloads" }
            } else {
                {downloads.iter().map(|download| {
                    rsx! {
                        DownloadItem { key: "{download.download_id}", client, download: download.clone() }
                    }
                })}
            }
        }
    }
}

#[component]
pub fn DownloadItem(client: Signal<Option<Client>>, download: Download) -> Element {
    let total: u64 = download.parts.iter().map(|p| p.total_bytes).sum();
    let downloaded: u64 = download.parts.iter().map(|p| p.bytes_downloaded).sum();
    let progress = if total > 0 {
        (downloaded as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    // temporary solution
    let status_copy = download.status.clone();
    rsx! {
        div { class: "download-item flex flex-column",
            div {
                class: "download-name flex items-center justify-between",
                div{"{download.filename}"},
                button{
                    class: "icon-button",

                    onclick: move |_|{
                        if let Some(client) = &mut *client.write() {
                            match client.send_and_receive(IpcRequest::ChangeDownloadStatus {
                                download_id: download.download_id.clone(),
                                download_status: match status_copy{
                                    DownloadStatus::Connecting => DownloadStatus::Paused,
                                    DownloadStatus::Downloading => DownloadStatus::Paused,
                                    _ => DownloadStatus::Downloading
                                } }) {
                                    Ok(response) => {
                                        match response {
                                            IpcResponse::Success => {
                                                println!("Download status changed");
                                            }
                                            _ => println!("Unexpected response"),
                                        }
                                        println!("Download status change REQUESTED")
                                    },
                                    Err(e) => println!("Error changing download status: {}", e),
                                }
                        }
                    },
                    img{
                        src: match download.status{
                            DownloadStatus::Connecting => PAUSE_ICON,
                            DownloadStatus::Downloading => PAUSE_ICON,
                            _ => PLAY_ICON
                        },
                        class: "icon",
                        alt: "Pause"
                    }
                }
            }
            div { class: "progress-bar", "data-progress": "70%", style: "--progress: {progress}%"}
            div { class: "flex items-center justify-between",
                div{"{format_bytes(downloaded)}/{format_bytes(total)}"},
                div{"{progress:.2}%"}
            }
        }
    }
}
