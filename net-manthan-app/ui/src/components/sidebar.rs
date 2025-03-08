use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum DownloadStatus {
    Downloading,
    Queued,
    Finished,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Clone, PartialEq)]
pub enum Pages {
    Downloads { status: Option<DownloadStatus> },
    Settings,
    About,
}

#[component]
pub fn Sidebar(current_page: Signal<Pages>) -> Element {
    let is_active = |page: Pages| -> bool { page == current_page() };

    let get_button_class = |page: Pages| -> String {
        if is_active(page) {
            "sidebar-button current-page".to_string()
        } else {
            "sidebar-button".to_string()
        }
    };

    rsx! {
        div {
            class: "sidebar flex flex-column",
            div {
                class: "download-options-wrapper flex flex-column",
                button {
                    class: get_button_class(Pages::Downloads { status: None }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: None }),
                    "All Downloads"
                }
                button {
                    class: get_button_class(Pages::Downloads { status: Some(DownloadStatus::Downloading) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Downloading) }),
                    "Active"
                }
                button {
                    class: get_button_class(Pages::Downloads { status: Some(DownloadStatus::Queued) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Queued) }),
                    "Queued"
                }
                button {
                    class: get_button_class(Pages::Downloads { status: Some(DownloadStatus::Finished) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Finished) }),
                    "Finished"
                }
                button {
                    class: get_button_class(Pages::Downloads { status: Some(DownloadStatus::Failed) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Failed) }),
                    "Failed"
                }
                button {
                    class: get_button_class(Pages::Downloads { status: Some(DownloadStatus::Cancelled) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Cancelled) }),
                    "Cancelled"
                }
                button {
                    class: get_button_class(Pages::Downloads { status: Some(DownloadStatus::Paused) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Paused) }),
                    "Paused"
                }
            }
            div {
                class: "other-options-wrapper flex flex-column",
                button {
                    class: get_button_class(Pages::Settings),
                    onclick: move |_| current_page.set(Pages::Settings),
                    "Settings"
                }
                button {
                    class: get_button_class(Pages::About),
                    onclick: move |_| current_page.set(Pages::About),
                    "About"
                }
            }
        }
    }
}
