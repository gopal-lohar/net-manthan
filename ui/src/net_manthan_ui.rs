use crate::components::side_bar::SideBar;
use crate::title_bar::TitleBar;

use gpui::{Context, Entity, IntoElement, Window, div, prelude::*, rgb};
use ui::PlatformStyle;

pub struct NetManthanUi {
    pub title_bar: Entity<TitleBar>,
    pub side_bar: Entity<SideBar>,
    pub platform_style: PlatformStyle,
}

impl Render for NetManthanUi {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0x000000))
            .w_full()
            .h_full()
            .when(
                !window.is_fullscreen()
                    && !window.is_maximized()
                    && match self.platform_style {
                        PlatformStyle::Linux => true,
                        _ => false,
                    },
                |d| d.border_1().border_color(rgb(0x30303f)),
            )
            .text_color(rgb(0xffffff))
            .child(self.title_bar.clone())
            .child(self.side_bar.clone())
    }
}

impl NetManthanUi {
    pub fn new(cx: &mut Context<NetManthanUi>) -> Self {
        let platform_style = PlatformStyle::platform();
        Self {
            title_bar: cx.new(|_| TitleBar::new()),
            side_bar: cx.new(|cx| SideBar::new(cx)),
            platform_style,
        }
    }
}
