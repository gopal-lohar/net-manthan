use crate::components::text_input::TextInput;
use gpui::{Entity, IntoElement, Window, div, hsla, prelude::*, rgb};
use ui::{Pixels, Rems};

pub struct AddDownloadDialog {
    url_input: Entity<TextInput>,
    title_bar_height: Pixels,
}

impl AddDownloadDialog {
    pub fn new(cx: &mut Context<Self>, title_bar_height: Pixels) -> Self {
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

        Self {
            url_input,
            title_bar_height,
        }
    }
}

impl Render for AddDownloadDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("add-download-dialog-backdrop")
            .absolute()
            .top(self.title_bar_height)
            .left_0()
            .w_full()
            .h_full()
            .bg(hsla(0., 0., 1., 0.01))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(move |_this, _ev, _window, cx| {
                cx.stop_propagation();
            }))
            .on_mouse_move(cx.listener(move |_this, _ev, _window, cx| {
                cx.stop_propagation();
            }))
            .child(
                div()
                    .bg(rgb(0x000000))
                    .border_1()
                    .mb_40()
                    .border_color(rgb(0x30303f))
                    .w(Rems(25.))
                    .child(div().px_4().py_2().text_center().child("Add Download"))
                    .child(
                        div()
                            .px_4()
                            .child(
                                div().child("URL").child(
                                    div()
                                        .border_1()
                                        .border_color(rgb(0xf0f0f0))
                                        .child(self.url_input.clone()),
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .py_4()
                                    .child(
                                        div()
                                            .px_4()
                                            .py_2()
                                            .bg(rgb(0xa00000))
                                            .rounded_md()
                                            .child("cancel"),
                                    )
                                    .child(
                                        div()
                                            .px_4()
                                            .py_2()
                                            .bg(rgb(0x00a000))
                                            .rounded_md()
                                            .child("Add"),
                                    ),
                            ),
                    ),
            )
    }
}
