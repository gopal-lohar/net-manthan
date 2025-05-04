use components::{net_manthan_ui::NetManthanUi, text_input::*};
use tracing::info;
use utils::logging::{self, Component, LogConfig};

use gpui::{
    App, Application, AssetSource, KeyBinding, SharedString, TitlebarOptions,
    WindowBackgroundAppearance, WindowKind, WindowOptions, point, prelude::*, px,
};
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub mod components;
pub mod platforms;

struct FsAssets {
    base: PathBuf,
}

impl AssetSource for FsAssets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        Ok(fs::read(self.base.join(path)).ok().map(Cow::Owned))
    }
    fn list(&self, _: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

fn main() {
    // Initialize logging
    match logging::init_logging(LogConfig {
        component: Component::Ui,
        log_dir: ".dev/logs".into(),
        silent_deps: vec!["naga".to_string(), "blade_graphics".to_string()],
        ..Default::default()
    }) {
        Ok(_) => {
            info!("Logger initialized for {}", Component::Ui.as_str());
        }
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
        }
    }

    // TODO: fix the path (dev or release?)
    let mut child = Command::new("./target/release/net-manthan")
        .spawn()
        .expect("Failed to start net-manthan");
    child.wait().expect("Process didn't finish");

    info!("Starting the GPUI application");
    Application::new()
        .with_assets(FsAssets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut App| {
            gpui_tokio::init(cx);
            cx.bind_keys([
                KeyBinding::new("backspace", Backspace, None),
                KeyBinding::new("delete", Delete, None),
                KeyBinding::new("left", Left, None),
                KeyBinding::new("right", Right, None),
                KeyBinding::new("shift-left", SelectLeft, None),
                KeyBinding::new("shift-right", SelectRight, None),
                KeyBinding::new("cmd-a", SelectAll, None),
                KeyBinding::new("cmd-v", Paste, None),
                KeyBinding::new("cmd-c", Copy, None),
                KeyBinding::new("cmd-x", Cut, None),
                KeyBinding::new("home", Home, None),
                KeyBinding::new("end", End, None),
                KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
            ]);

            // TODO: Start the server/daemon
            // TODO: Initialize the settings and adapt some things like default window size etc.
            let window = cx
                .open_window(
                    WindowOptions {
                        titlebar: Some(TitlebarOptions {
                            title: None,
                            appears_transparent: true,
                            traffic_light_position: Some(point(px(9.0), px(9.0))),
                        }),
                        window_bounds: None,
                        focus: true,
                        show: true,
                        kind: WindowKind::Normal,
                        is_movable: true,
                        window_background: WindowBackgroundAppearance::Transparent,
                        window_decorations: Some(gpui::WindowDecorations::Client),
                        window_min_size: Some(gpui::Size {
                            width: px(360.0),
                            height: px(240.0),
                        }),
                        ..Default::default()
                    },
                    |window, cx| cx.new(|cx| NetManthanUi::new(window, cx)),
                )
                .unwrap();

            cx.on_keyboard_layout_change({
                move |cx| {
                    window.update(cx, |_, _, cx| cx.notify()).ok();
                }
            })
            .detach();
        });
}
