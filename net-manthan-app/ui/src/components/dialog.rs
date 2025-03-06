use dioxus::prelude::*;
use download_engine::types::{IpcRequest, IpcResponse};
use utils::Client;

#[component]
pub fn Dialog(client: Signal<Option<Client>>, show_dialog: Signal<bool>) -> Element {
    let mut url = use_signal(String::new);

    rsx! {
        div {
            class: "dialog-root",
            onclick: move |_| {
                *show_dialog.write() = false
            },

            div {
                class: "dialog-content",
                onclick: move |evt| {
                    evt.stop_propagation();
                },
                div {
                    class: "dialog-header flex justify-center items-center",
                    h2 { "Download" }
                }
                div {
                    class: "dialog-body",
                    input {
                        class: "url-input",
                        placeholder: "Enter download URL",
                        value: "{url}",
                        oninput: move |evt| url.set(evt.value().clone()),

                    }
                    div {
                        class: "dialog-buttons flex justify-between",
                        button { onclick: move |_| *show_dialog.write() = false, "Cancel"}
                        button { onclick: move |_| {
                            url.with_mut(|current_url| {
                                if let Some(client) = &mut *client.write() {
                                    match client.send_and_receive(IpcRequest::StartDownload { url: current_url.to_string(), output_path: None, thread_count: None, headers: None }) {
                                        Ok(IpcResponse::Success) => {
                                            println!("yeah, download started");
                                        }
                                        Ok(_) => {
                                            println!("Received unexpected response type");
                                        }
                                        Err(err) => {
                                            println!("Failed to fetch downloads: {}", err);
                                        }
                                    }
                                }else{
                                    println!("Client not initialized");
                                }
                                *current_url = String::new();
                                *show_dialog.write() = false
                            });
                        },"Download" }
                    }
                }
            }
        }
    }
}
