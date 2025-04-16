use gpui::{
    App, Application, TitlebarOptions, WindowBackgroundAppearance, WindowKind, WindowOptions,
    point, prelude::*, px,
};

use crate::components::side_bar::SideBar;
use net_manthan_ui::NetManthanUi;
use title_bar::TitleBar;

pub mod components;
mod net_manthan_ui;
pub mod platforms;
pub mod title_bar;
pub mod window_controls;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_tokio::init(cx);
        // TODO: Initialize logging
        // TODO: Start the server/daemon
        // TODO: Initialize the settings and adapt some things like default window size etc.
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(9.0), px(9.0))),
                }),
                window_bounds: None,
                focus: false,
                show: false,
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
            |_, cx| {
                cx.new(|cx| NetManthanUi {
                    title_bar: cx.new(|_| TitleBar::new()),
                    side_bar: cx.new(|_| SideBar::new()),
                })
            },
        )
        .unwrap();
    });
}
