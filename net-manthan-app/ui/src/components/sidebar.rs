use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum DownloadStatus {
    Downloading,
    Queued,
    Finished,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Clone, PartialEq)]
enum Pages {
    Downloads { status: Option<DownloadStatus> },
    Settings,
    About,
}

// for maintainabibiblbiety
impl Pages {
    fn display_text(&self) -> &'static str {
        match self {
            Pages::Downloads { status: None } => "All Downloads",
            Pages::Downloads {
                status: Some(DownloadStatus::Downloading),
            } => "Active",
            Pages::Downloads {
                status: Some(DownloadStatus::Queued),
            } => "Queued",
            Pages::Downloads {
                status: Some(DownloadStatus::Finished),
            } => "Finished",
            Pages::Downloads {
                status: Some(DownloadStatus::Failed),
            } => "Failed",
            Pages::Downloads {
                status: Some(DownloadStatus::Cancelled),
            } => "Cancelled",
            Pages::Downloads {
                status: Some(DownloadStatus::Paused),
            } => "Paused",
            Pages::Settings => "Settings",
            Pages::About => "About",
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    let mut current_page = use_signal(|| Pages::Downloads { status: None });

    let is_active = |page: &Pages| -> bool { page == &*current_page.read() };

    let get_button_class = |page: &Pages| -> String {
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
                    class: get_button_class(&Pages::Downloads { status: None }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: None }),
                    "All Downloads"
                }
                button {
                    class: get_button_class(&Pages::Downloads { status: Some(DownloadStatus::Downloading) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Downloading) }),
                    "Active"
                }
                button {
                    class: get_button_class(&Pages::Downloads { status: Some(DownloadStatus::Queued) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Queued) }),
                    "Queued"
                }
                button {
                    class: get_button_class(&Pages::Downloads { status: Some(DownloadStatus::Finished) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Finished) }),
                    "Finished"
                }
                button {
                    class: get_button_class(&Pages::Downloads { status: Some(DownloadStatus::Failed) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Failed) }),
                    "Failed"
                }
                button {
                    class: get_button_class(&Pages::Downloads { status: Some(DownloadStatus::Cancelled) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Cancelled) }),
                    "Cancelled"
                }
                button {
                    class: get_button_class(&Pages::Downloads { status: Some(DownloadStatus::Paused) }),
                    onclick: move |_| current_page.set(Pages::Downloads { status: Some(DownloadStatus::Paused) }),
                    "Paused"
                }
            }
            div {
                class: "other-options-wrapper flex flex-column",
                button {
                    class: get_button_class(&Pages::Settings),
                    onclick: move |_| current_page.set(Pages::Settings),
                    "Settings"
                }
                button {
                    class: get_button_class(&Pages::About),
                    onclick: move |_| current_page.set(Pages::About),
                    "About"
                }
            }
        }
    }
}
