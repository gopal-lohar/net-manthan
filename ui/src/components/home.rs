use std::path::Path;

use download_engine::Download;
use gpui::{AsyncApp, ClipboardItem, IntoElement, WeakEntity, Window, div, prelude::*, rgb};
use std::time::Duration;
use ui::{DefiniteLength, ParentElement, SharedString};
use utils::rpc::{NativeRpcSettings, client::NativeRpcClient};

pub struct Home {
    downloads: Vec<Download>,
}

impl Home {
    pub fn new(cx: &mut Context<Home>) -> Self {
        cx.spawn(|this: WeakEntity<Home>, cx: &mut AsyncApp| {
            // Clone the app context before moving into async block
            let app_context = cx.clone();
            async move {
                loop {
                    println!("before hihihihihih");

                    // fetch data
                    let downloads = gpui_tokio::Tokio::spawn(&app_context, async move {
                        let client = NativeRpcClient::connect(&NativeRpcSettings {
                            address: "hellyeah".into(),
                            allow_all_users: true,
                            secret: "".into(),
                        })
                        .await;
                        let downloads = match client {
                            Ok(mut client) => {
                                let response = match client.get_downloads().await {
                                    Ok(downloads) => Some(downloads),
                                    Err(err) => {
                                        println!("Error: {}", err);
                                        None
                                    }
                                };
                                match client.close().await {
                                    Ok(()) => println!("Client closed successfully"),
                                    Err(err) => println!("Error closing client: {}", err),
                                };
                                response
                            }
                            Err(err) => {
                                println!("Error: {}", err);
                                None
                            }
                        };
                        downloads
                    });

                    let data = match downloads {
                        Ok(downloads) => match downloads.await {
                            Ok(downloads) => downloads,
                            Err(err) => {
                                println!("Error: {}", err);
                                None
                            }
                        },
                        Err(err) => {
                            println!("Error: {}", err);
                            None
                        }
                    };

                    // Schedule update on main thread
                    app_context
                        .update(|cx| {
                            if let Some(strong_this) = this.upgrade() {
                                strong_this.update(cx, |downloads, cx| {
                                    match data {
                                        Some(data) => {
                                            downloads.downloads = data;
                                        }
                                        None => {}
                                    }
                                    cx.notify();
                                });
                            }
                        })
                        .ok(); // Handle potential error

                    app_context
                        .background_executor()
                        .timer(Duration::from_millis(1000))
                        .await;
                    println!("hihihihihi");
                }
            }
        })
        .detach();

        Self { downloads: vec![] }
    }
}

impl Render for Home {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().children(self.downloads.iter().map(|download| {
            div()
                .border_1()
                .rounded_md()
                .border_color(rgb(0x202030))
                .p_4()
                .child(
                    download
                        .file_name
                        .clone()
                        .unwrap_or("default-filename.nm".into())
                        .into_os_string()
                        .into_string()
                        .unwrap_or("default-filename.nm".into()),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .id(SharedString::new("open-directory"))
                                .cursor_pointer()
                                .rounded_md()
                                .px_2()
                                .child("Open Location")
                                .bg(rgb(0x202030))
                                .on_click({
                                    let file_path = download.file.clone();
                                    cx.listener(move |_this, _ev, _window, cx| {
                                        cx.reveal_path(Path::new(&file_path));
                                    })
                                }),
                        )
                        .child(
                            div()
                                .id(SharedString::new("copy-download-link"))
                                .cursor_pointer()
                                .rounded_md()
                                .px_2()
                                .child("Copy Download Link")
                                .bg(rgb(0x202030))
                                .on_click({
                                    let download_url = download.url.clone();
                                    cx.listener(move |_this, _ev, _window, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            download_url.to_string(),
                                        ));
                                    })
                                }),
                        ),
                )
                .child(
                    div()
                        .mt_2()
                        .flex()
                        .justify_between()
                        .child(div().child(format!("{:.2}%", download.get_progress_percentage())))
                        .child(div().child(format!(
                            "{}/{}",
                            format_bytes(download.get_bytes_downloaded()),
                            format_bytes(download.get_total_size())
                        ))),
                )
                .child(
                    div()
                        .mt_1()
                        .h_2()
                        .w_full()
                        .bg(rgb(0x202030))
                        .rounded_full()
                        .child(div().bg(rgb(0xff00ff)).h_full().rounded_full().w(
                            DefiniteLength::Fraction(
                                (download.get_progress_percentage() as f32) / 100.0,
                            ),
                        )),
                )
        }))
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    let rounded = (size * 10.0).round() / 10.0;
    let formatted = format!("{:.2}", rounded);

    let cleaned = if formatted.ends_with(".0") {
        &formatted[..formatted.len() - 2]
    } else {
        &formatted
    };

    format!("{}{}", cleaned, UNITS[unit_index])
}
