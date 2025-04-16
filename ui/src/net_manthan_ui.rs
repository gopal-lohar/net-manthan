use crate::components::side_bar::SideBar;
use crate::title_bar::TitleBar;

use gpui::{Context, Entity, IntoElement, SharedString, Window, div, prelude::*, rgb};

pub struct NetManthanUi {
    pub text: SharedString,
    pub title_bar: Entity<TitleBar>,
    pub side_bar: Entity<SideBar>,
}

impl Render for NetManthanUi {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0x000000))
            .w_full()
            .h_full()
            .border_1()
            .border_color(rgb(0x30303f))
            .text_color(rgb(0xffffff))
            .child(self.title_bar.clone())
            .child(self.side_bar.clone())
    }
}
