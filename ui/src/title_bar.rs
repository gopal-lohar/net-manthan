use gpui::{IntoElement, Window, div, prelude::*, rgb};

pub struct TitleBar {
    should_move: bool,
}

impl TitleBar {
    pub fn new() -> Self {
        Self { should_move: false }
    }
}

impl Render for TitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_10()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(rgb(0x30303f))
            .bg(if window.is_window_active() && !self.should_move {
                rgb(0x000000)
            } else {
                rgb(0x101020)
            })
            .on_mouse_move(cx.listener(move |this, _ev, window, _| {
                if this.should_move {
                    this.should_move = false;
                    window.start_window_move();
                }
            }))
            .on_mouse_down_out(cx.listener(move |this, _ev, _window, _cx| {
                println!("down down down down down");
                this.should_move = false;
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(move |this, _ev, _window, _cx| {
                    this.should_move = false;
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _ev, _window, _cx| {
                    this.should_move = true;
                }),
            )
            .child(div().px_4().child("Net Manthan"))
    }
}
