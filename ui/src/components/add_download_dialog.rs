use gpui::{IntoElement, Window, div, hsla, prelude::*, rgb};
use ui::Rems;

pub struct AddDownloadDialog {}

impl AddDownloadDialog {
    pub fn new() -> Self {
        Self {}
    }
}

impl Render for AddDownloadDialog {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .bg(hsla(0., 0., 1., 0.01))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .bg(rgb(0x000000))
                    .border_1()
                    .border_color(rgb(0x30303f))
                    .max_w(Rems(25.))
                    .child(div().px_4().px_2().child("Add Download")),
            )
    }
}
