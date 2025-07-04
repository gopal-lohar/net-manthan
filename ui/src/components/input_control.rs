use crate::Theme;
use crate::components::input::TextInput;
use crate::helpers::icon::{Icon, IconName};
use gpui::*;

pub struct InputControl {
    text_input: Entity<TextInput>,
}

impl InputControl {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|app| InputControl {
            text_input: app.new(|cx| TextInput {
                focus_handle: cx.focus_handle(),
                content: "".into(),
                placeholder: "Add todo...".into(),
                selected_range: 0..0,
                selection_reversed: false,
                marked_range: None,
                last_layout: None,
                last_bounds: None,
                is_selecting: false,
            }),
        })
    }
    fn submit(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.text_input
            .update(cx, |text_input, _cx| text_input.reset());
        cx.notify();
    }
}

impl Render for InputControl {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let input = div()
            .flex()
            .flex_grow()
            .p_1()
            .rounded_md()
            .bg(theme.mantle)
            .border_1()
            .border_color(theme.crust)
            .child(self.text_input.clone());

        let button = div()
            .flex()
            .justify_center()
            .items_center()
            .p_1()
            .bg(theme.surface0)
            .min_w(px(42.0))
            .rounded_md()
            .cursor_pointer()
            .hover(|x| x.bg(theme.surface1))
            .border_color(theme.crust)
            .border_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(IconName::Plus)),
            )
            .on_mouse_down(MouseButton::Left, cx.listener(Self::submit));

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(div().flex().gap_1().mt(px(10.)).child(input).child(button))
    }
}
