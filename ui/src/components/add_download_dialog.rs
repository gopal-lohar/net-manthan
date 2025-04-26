use crate::components::text_input::TextInput;
use gpui::{Entity, IntoElement, Window, div, hsla, prelude::*, rgb};
use ui::Rems;

pub struct AddDownloadDialog {
    url_input: Entity<TextInput>,
}

impl AddDownloadDialog {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let url_input = cx.new(|cx| TextInput {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "Enter URL to download...".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        });

        Self { url_input }
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
                    .child(
                        div()
                            .px_4()
                            .px_2()
                            .child("Add Download")
                            .child(self.url_input.clone()),
                    ),
            )
    }
}
