use gpui::{MouseButton, prelude::*};

use ui::prelude::*;

use crate::components::title_bar::window_controls::{WindowControl, WindowControlType};

#[derive(IntoElement)]
pub struct LinuxWindowControls {}

impl LinuxWindowControls {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderOnce for LinuxWindowControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .id("generic-window-controls")
            .px_3()
            .gap_3()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(WindowControl::new(
                "minimize",
                WindowControlType::Minimize,
                cx,
            ))
            .child(WindowControl::new(
                "maximize-or-restore",
                if window.is_maximized() {
                    WindowControlType::Restore
                } else {
                    WindowControlType::Maximize
                },
                cx,
            ))
            .child(WindowControl::new_close(
                "close",
                WindowControlType::Close,
                cx,
            ))
    }
}
