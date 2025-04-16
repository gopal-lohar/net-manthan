use gpui::{
    Decorations, IntoElement, Pixels, Rgba, Window, WindowAppearance, div, prelude::*, prelude::*,
    px, rgb,
};

use ui::PlatformStyle;
use ui::prelude::*;

pub struct TitleBar {
    should_move: bool,
    platform_style: PlatformStyle,
}

impl TitleBar {
    pub fn new() -> Self {
        let platform_style = PlatformStyle::platform();
        Self {
            should_move: false,
            platform_style,
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn height(window: &mut Window) -> Pixels {
        (1.75 * window.rem_size()).max(px(34.))
    }

    #[cfg(target_os = "windows")]
    pub fn height(_window: &mut Window) -> Pixels {
        // todo(windows) instead of hard coded size report the actual size to the Windows platform API
        px(32.)
    }

    /// Sets the platform style.
    pub fn platform_style(mut self, style: PlatformStyle) -> Self {
        self.platform_style = style;
        self
    }
}

impl Render for TitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let decorations = window.window_decorations();
        let height = Self::height(window);

        div()
            .h_10()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(rgb(0x30303f))
            .bg(if window.is_window_active() && !self.should_move {
                rgb(0x000000)
            } else {
                rgb(0x101020)
            })
            .child(div().px_4().child("Net Manthan"))
            .when(!window.is_fullscreen(), |title_bar| {
                match self.platform_style {
                    PlatformStyle::Mac => title_bar,
                    PlatformStyle::Linux => {
                        if matches!(decorations, Decorations::Client { .. }) {
                            title_bar
                                .child("asfasdf")
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
                        } else {
                            title_bar
                        }
                    }
                    PlatformStyle::Windows => title_bar.child(WindowsWindowControls::new(height)),
                }
            })
    }
}

#[derive(IntoElement)]
pub struct WindowsWindowControls {
    button_height: Pixels,
}

impl WindowsWindowControls {
    pub fn new(button_height: Pixels) -> Self {
        Self { button_height }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_font() -> &'static str {
        "Segoe Fluent Icons"
    }

    #[cfg(target_os = "windows")]
    fn get_font() -> &'static str {
        use windows::Wdk::System::SystemServices::RtlGetVersion;

        let mut version = unsafe { std::mem::zeroed() };
        let status = unsafe { RtlGetVersion(&mut version) };

        if status.is_ok() && version.dwBuildNumber >= 22000 {
            "Segoe Fluent Icons"
        } else {
            "Segoe MDL2 Assets"
        }
    }
}

impl RenderOnce for WindowsWindowControls {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let close_button_hover_color = Rgba {
            r: 232.0 / 255.0,
            g: 17.0 / 255.0,
            b: 32.0 / 255.0,
            a: 1.0,
        };

        let button_hover_color = match window.appearance() {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Rgba {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 0.2,
            },
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Rgba {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 0.1,
            },
        };

        div()
            .id("windows-window-controls")
            .font_family(Self::get_font())
            .flex()
            .flex_row()
            .justify_center()
            .content_stretch()
            .max_h(self.button_height)
            .min_h(self.button_height)
            .child(WindowsCaptionButton::new(
                "minimize",
                WindowsCaptionButtonIcon::Minimize,
                button_hover_color,
            ))
            .child(WindowsCaptionButton::new(
                "maximize-or-restore",
                if window.is_maximized() {
                    WindowsCaptionButtonIcon::Restore
                } else {
                    WindowsCaptionButtonIcon::Maximize
                },
                button_hover_color,
            ))
            .child(WindowsCaptionButton::new(
                "close",
                WindowsCaptionButtonIcon::Close,
                close_button_hover_color,
            ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum WindowsCaptionButtonIcon {
    Minimize,
    Restore,
    Maximize,
    Close,
}

#[derive(IntoElement)]
struct WindowsCaptionButton {
    id: ElementId,
    icon: WindowsCaptionButtonIcon,
    hover_background_color: Rgba,
}

impl WindowsCaptionButton {
    pub fn new(
        id: impl Into<ElementId>,
        icon: WindowsCaptionButtonIcon,
        hover_background_color: Rgba,
    ) -> Self {
        Self {
            id: id.into(),
            icon,
            hover_background_color,
        }
    }
}

impl RenderOnce for WindowsCaptionButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // todo(windows) report this width to the Windows platform API
        // NOTE: this is intentionally hard coded. An option to use the 'native' size
        //       could be added when the width is reported to the Windows platform API
        //       as this could change between future Windows versions.
        let width = px(36.);

        h_flex()
            .id(self.id)
            .justify_center()
            .content_center()
            .w(width)
            .h_full()
            .text_size(px(10.0))
            .hover(|style| style.bg(self.hover_background_color))
            .active(|style| {
                let mut active_color = self.hover_background_color;
                active_color.a *= 0.2;

                style.bg(active_color)
            })
            .child(match self.icon {
                WindowsCaptionButtonIcon::Minimize => "\u{e921}",
                WindowsCaptionButtonIcon::Restore => "\u{e923}",
                WindowsCaptionButtonIcon::Maximize => "\u{e922}",
                WindowsCaptionButtonIcon::Close => "\u{e8bb}",
            })
    }
}
