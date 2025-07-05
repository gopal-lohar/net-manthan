use crate::helpers::icon::{Icon, IconName};
use gpui::{App, ElementId, Hsla, Window, div, prelude::*};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum WindowControlType {
    Minimize,
    Restore,
    Maximize,
    Close,
}

#[allow(unused)]
pub struct WindowControlStyle {
    background: Hsla,
    background_hover: Hsla,
    icon: Hsla,
    icon_hover: Hsla,
}

impl WindowControlStyle {
    pub fn default(_cx: &mut App) -> Self {
        Self {
            background: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.,
                a: 1.,
            },
            background_hover: Hsla {
                h: 0.,
                s: 0.,
                l: 1.,
                a: 0.025,
            },
            icon: Hsla {
                h: 0.,
                s: 0.,
                l: 0.75,
                a: 1.0,
            },
            icon_hover: Hsla {
                h: 0.,
                s: 0.,
                l: 1.,
                a: 1.0,
            },
        }
    }

    #[allow(unused)]
    /// Sets the background color of the control.
    pub fn background(mut self, color: impl Into<Hsla>) -> Self {
        self.background = color.into();
        self
    }

    #[allow(unused)]
    /// Sets the background color of the control when hovered.
    pub fn background_hover(mut self, color: impl Into<Hsla>) -> Self {
        self.background_hover = color.into();
        self
    }

    #[allow(unused)]
    /// Sets the color of the icon.
    pub fn icon(mut self, color: impl Into<Hsla>) -> Self {
        self.icon = color.into();
        self
    }

    #[allow(unused)]
    /// Sets the color of the icon when hovered.
    pub fn icon_hover(mut self, color: impl Into<Hsla>) -> Self {
        self.icon_hover = color.into();
        self
    }
}

#[derive(IntoElement)]
pub struct WindowControl {
    id: ElementId,
    icon: WindowControlType,
    style: WindowControlStyle,
}

impl WindowControl {
    pub fn new(id: impl Into<ElementId>, icon: WindowControlType, cx: &mut App) -> Self {
        let style = WindowControlStyle::default(cx);

        Self {
            id: id.into(),
            icon,
            style,
        }
    }

    pub fn new_close(id: impl Into<ElementId>, icon: WindowControlType, cx: &mut App) -> Self {
        let style = WindowControlStyle::default(cx);

        Self {
            id: id.into(),
            icon,
            style,
        }
    }

    #[allow(unused)]
    pub fn custom_style(
        id: impl Into<ElementId>,
        icon: WindowControlType,
        style: WindowControlStyle,
    ) -> Self {
        Self {
            id: id.into(),
            icon,
            style,
        }
    }
}

impl RenderOnce for WindowControl {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let icon = Icon::new(match self.icon {
            WindowControlType::Minimize => IconName::Minimize,
            WindowControlType::Restore => IconName::Restore,
            WindowControlType::Maximize => IconName::Maximize,
            WindowControlType::Close => IconName::Close,
        })
        .text_color(self.style.icon);

        div()
            .id(self.id)
            .group("")
            .cursor_pointer()
            .justify_center()
            .content_center()
            .rounded_2xl()
            .size_7()
            .hover(|this| this.bg(self.style.background_hover))
            .active(|this| this.bg(self.style.background_hover))
            .flex()
            .items_center()
            .justify_center()
            .child(icon)
            .on_mouse_move(|_, _, cx| cx.stop_propagation())
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                match self.icon {
                    WindowControlType::Minimize => window.minimize_window(),
                    WindowControlType::Restore => window.zoom_window(),
                    WindowControlType::Maximize => window.zoom_window(),
                    WindowControlType::Close => window.remove_window(),
                }
            })
    }
}
