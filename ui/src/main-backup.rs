use std::borrow::Cow;

use gpui::{
    App, Application, AssetSource, TitlebarOptions, WindowBackgroundAppearance, WindowKind,
    WindowOptions, point, prelude::*, px,
};

use components::net_manthan_ui::NetManthanUi;
use std::fs;
use std::path::PathBuf;

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
    Application::new()
        .with_assets(FsAssets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut App| {
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
        });
}
