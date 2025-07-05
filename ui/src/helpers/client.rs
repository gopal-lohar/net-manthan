use download_engine::Download;
use gpui::*;
use gpui_tokio::Tokio;
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::rpc::{NativeRpcSettings, client::NativeRpcClient};

pub enum Handle {
    Connecting,
    Connected(Arc<Mutex<NativeRpcClient>>),
    Failed,
}

pub struct Client {
    pub handle: Entity<Handle>,
}

impl Client {
    pub fn init(app: &mut App, settings: NativeRpcSettings) {
        let handle = app.new(|_| Handle::Connecting);
        app.set_global(Client { handle });
        app.spawn(move |app: &mut AsyncApp| {
            let mut app = app.clone();
            async move {
                let thread_handle = Tokio::spawn(&app, async move {
                    let client = NativeRpcClient::connect(&settings).await;
                    match client {
                        Ok(client) => Handle::Connected(Arc::new(Mutex::new(client))),
                        Err(_) => Handle::Failed,
                    }
                });
                let handle = match thread_handle {
                    Ok(h) => match h.await {
                        Ok(handle) => handle,
                        Err(_) => Handle::Failed,
                    },
                    Err(_) => Handle::Failed,
                };

                Client::update(
                    |this, cx| {
                        this.handle.update(cx, |h, _| {
                            *h = handle;
                        })
                    },
                    &mut app,
                );
            }
        })
        .detach();
    }

    pub async fn get_downloads(cx: &mut AsyncApp) -> Vec<Download> {
        let rpc_client = cx.try_read_global(|client: &Client, cx: &App| {
            let handle = client.handle.read(cx);
            if let Handle::Connected(rpc_client) = handle {
                return Some(rpc_client.clone());
            } else {
                return None;
            }
        });

        match rpc_client.unwrap_or(None) {
            Some(rpc_client) => {
                let cx = cx.clone();
                let thread_handle = Tokio::spawn(&cx, {
                    let client = rpc_client.clone();
                    async move {
                        let mut client = client.lock().await;
                        match client.get_downloads().await {
                            Ok(downloads) => Some(downloads),
                            Err(_) => None,
                        }
                    }
                });

                match thread_handle {
                    Ok(h) => match h.await {
                        Ok(d) => return d.unwrap_or(vec![]),
                        Err(_) => {}
                    },
                    Err(_) => {}
                }
            }
            None => {}
        }

        vec![]
    }

    pub fn update(f: impl FnOnce(&mut Self, &mut App), cx: &mut AsyncApp) {
        if !cx.has_global::<Self>().unwrap_or(false) {
            return;
        }
        let _ = cx.update_global::<Self, _>(|mut this, cx| {
            f(&mut this, cx);
        });
    }
}

impl Global for Client {}
