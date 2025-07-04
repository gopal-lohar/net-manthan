use download_engine::Download;
use gpui::*;
use gpui_tokio::Tokio;
use utils::rpc::{NativeRpcSettings, client::NativeRpcClient};

pub enum Handle {
    Connecting,
    Connected(NativeRpcClient),
    Failed,
}

pub struct Client {
    handle: Entity<Handle>,
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
                        Ok(client) => Handle::Connected(client),
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

    pub async fn get_downloads(&mut self, cx: &mut AsyncApp) -> Vec<Download> {
        let downloads = cx.update_entity(&self.handle, move |handle, _| async {
            let client = match handle {
                Handle::Connected(client) => client,
                _ => return vec![],
            };
            let cx = cx.clone();
            let thread_handle = Tokio::spawn(&cx, {
                async move {
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
            return vec![];
        });

        match downloads {
            Ok(downloads) => downloads.await,
            Err(_) => vec![],
        }
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
