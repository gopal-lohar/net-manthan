use gpui::{Hsla, svg};
use ui::prelude::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum WindowControlType {
    Minimize,
    Restore,
    Maximize,
    Close,
}

impl WindowControlType {
    /// Returns the icon name for the window control type.
    ///
    /// Will take a [PlatformStyle] in the future to return a different
    /// icon name based on the platform.
    pub fn icon(&self) -> IconName {
        match self {
            WindowControlType::Minimize => IconName::GenericMinimize,
            WindowControlType::Restore => IconName::GenericRestore,
            WindowControlType::Maximize => IconName::GenericMaximize,
            WindowControlType::Close => IconName::GenericClose,
        }
    }
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
                s: 1.0,
                l: 0.5,
                a: 1.0,
            },
            background_hover: Hsla {
                h: 0.2,
                s: 1.0,
                l: 0.5,
                a: 1.0,
            },
            icon: Hsla {
                h: 0.4,
                s: 1.0,
                l: 0.5,
                a: 1.0,
            },
            icon_hover: Hsla {
                h: 0.6,
                s: 1.0,
                l: 0.5,
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
        let icon = svg()
            .size_4()
            .flex_none()
            .path(match self.icon {
                WindowControlType::Minimize => "<svg width='16' height='16' viewBox='0 0 16 16' fill='none' xmlns='http://www.w3.org/2000/svg'><path d='M9.5 6.5H3.5V12.5H9.5V6.5Z' stroke='#FBF1C7'/><path d='M10 8.5L12.5 8.5L12.5 3.5L7.5 3.5L7.5 6' stroke='#FBF1C7'/></svg>",
                WindowControlType::Restore => "<svg width='16' height='16' viewBox='0 0 16 16' fill='none' xmlns='http://www.w3.org/2000/svg'><path d='M9.5 6.5H3.5V12.5H9.5V6.5Z' stroke='#FBF1C7'/><path d='M10 8.5L12.5 8.5L12.5 3.5L7.5 3.5L7.5 6' stroke='#FBF1C7'/></svg>",
                WindowControlType::Maximize => "<svg width='16' height='16' viewBox='0 0 16 16' fill='none' xmlns='http://www.w3.org/2000/svg'><path d='M9.5 6.5H3.5V12.5H9.5V6.5Z' stroke='#FBF1C7'/><path d='M10 8.5L12.5 8.5L12.5 3.5L7.5 3.5L7.5 6' stroke='#FBF1C7'/></svg>",
                WindowControlType::Close => "<svg width='16' height='16' viewBox='0 0 16 16' fill='none' xmlns='http://www.w3.org/2000/svg'><path d='M9.5 6.5H3.5V12.5H9.5V6.5Z' stroke='#FBF1C7'/><path d='M10 8.5L12.5 8.5L12.5 3.5L7.5 3.5L7.5 6' stroke='#FBF1C7'/></svg>",
            })
            .text_color(self.style.icon)
            .group_hover("", |this| this.text_color(self.style.icon_hover));

        h_flex()
            .id(self.id)
            .group("")
            .cursor_pointer()
            .justify_center()
            .content_center()
            .rounded_2xl()
            .w_5()
            .h_5()
            .hover(|this| this.bg(self.style.background_hover))
            .active(|this| this.bg(self.style.background_hover))
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
