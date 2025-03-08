use dioxus::prelude::*;
#[derive(Clone, PartialEq)]
pub enum Pages {
    Downloading,
    Queued,
    Stopped,
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
                    class: get_button_class(Pages::Downloading),
                    onclick: move |_| current_page.set(Pages::Downloading),
                    "Downloading"
                }
                button {
                    class: get_button_class(Pages::Queued),
                    onclick: move |_| current_page.set(Pages::Queued),
                    "Queued"
                }
                button {
                    class: get_button_class(Pages::Stopped),
                    onclick: move |_| current_page.set(Pages::Stopped),
                    "Stopped"
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
