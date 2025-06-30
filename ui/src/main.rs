use crate::helpers::{
    assets::Assets,
    theme::Theme,
    window::{blur_window, get_window_options},
};
use components::{net_manthan_ui::NetManthanUi, text_input::*};
use gpui::{App, Application, KeyBinding, prelude::*};
use std::process::Command;
use tracing::info;
use utils::logging::{Component, get_ui_config, init_logging};

pub mod components;
pub mod helpers;
pub mod platforms;

#[cfg(not(target_os = "windows"))]
fn net_manthan_path() -> &'static str {
    "/home/titan/code/net-manthan/target/debug/net-manthan"
}

#[cfg(target_os = "windows")]
fn net_manthan_path() -> &'static str {
    ".\\target\\debug\\net-manthan.exe"
}

fn main() {
    // Initialize logging
    match init_logging(get_ui_config(".dev/logs")) {
        Ok(_) => {
            info!("Logger initialized for {}", Component::Ui.as_str());
        }
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
        }
    }

    // NOTE: ui/build.rs builds debug and we run debug here
    let mut _child = Command::new(net_manthan_path())
        .arg("--daemon")
        .arg("http://localhost:8080/slow/extra_large.bin")
        .spawn()
        .expect("Failed to start net-manthan");

    info!("Starting the GPUI application");
    Application::new().with_assets(Assets).run(|cx: &mut App| {
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

        // TODO: Initialize the settings and adapt some things like default window size etc.
        let window_options = get_window_options(cx);
        let window = cx
            .open_window(window_options, |window, app| {
                blur_window(window);
                Theme::init(app);
                app.new(|cx| NetManthanUi::new(window, cx))
            })
            .unwrap();

        cx.on_keyboard_layout_change({
            move |cx| {
                window.update(cx, |_, _, cx| cx.notify()).ok();
            }
        })
        .detach();
    });
}
