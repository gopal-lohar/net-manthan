use std::path::Path;

use gpui::{ClipboardItem, IntoElement, PromptLevel, Window, div, prelude::*, rgb};
use ui::{DefiniteLength, ParentElement, SharedString};

struct Download {
    file_name: String,
    file_path: String,
    download_url: String,
    size_downloaded: u64,
    total_size: u64,
}

impl Download {
    fn download_fraction(&self) -> f32 {
        return self.size_downloaded as f32 / self.total_size as f32;
    }
}

pub struct Home {
    downloads: Vec<Download>,
}

impl Home {
    pub fn new() -> Self {
        Self {
            downloads: vec![Download {
                file_name: String::from("mediacreationtool.exe"),
                file_path: String::from("/home/charon/Downloads/mediacreationtool.exe"),
                download_url: String::from(
                    "https://software-static.download.prss.microsoft.com/dbazure/888969d5-f34g-4e03-ac9d-1f9786c66749/mediacreationtool.exe",
                ),
                size_downloaded: 516177408,
                total_size: 716177408,
            }],
        }
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
                .child(download.file_name.clone())
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
                                    let file_path = download.file_path.clone();
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
                                    let download_url = download.download_url.clone();
                                    cx.listener(move |_this, _ev, _window, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            download_url.to_string(),
                                        ));
                                    })
                                }),
                        )
                        .child(
                            div()
                                .id(SharedString::new("do-download-link"))
                                .cursor_pointer()
                                .rounded_md()
                                .px_2()
                                .child("Do Download")
                                .bg(rgb(0x202030))
                                .on_click({
                                    let download_url = download.download_url.clone();
                                    cx.listener(move |_this, _ev, window, cx| {
                                        let _ = window.prompt(
                                            PromptLevel::Info,
                                            "message",
                                            Some("sadfasdfasdfasdfsdfsdfsdf"),
                                            &["asdf"],
                                            cx,
                                        );
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
                        .child(div().child(format!("{:.2}%", download.download_fraction() * 100.0)))
                        .child(div().child(format!(
                            "{}/{}",
                            format_bytes(download.size_downloaded),
                            format_bytes(download.total_size)
                        ))),
                )
                .child(
                    div()
                        .mt_1()
                        .h_2()
                        .w_full()
                        .bg(rgb(0x202030))
                        .rounded_full()
                        .child(
                            div()
                                .bg(rgb(0xff00ff))
                                .h_full()
                                .rounded_full()
                                .w(DefiniteLength::Fraction(download.download_fraction())),
                        ),
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
