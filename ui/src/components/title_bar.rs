use gpui::{Decorations, IntoElement, Pixels, Window, div, px, rgb};

use ui::PlatformStyle;
use ui::prelude::*;

use crate::platforms::platform_linux::LinuxWindowControls;
use crate::platforms::platform_windows::WindowsWindowControls;

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
        (2. * window.rem_size()).max(px(34.))
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
            .h(height + Pixels(1.))
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
                                .child(LinuxWindowControls::new())
                                .on_mouse_move(cx.listener(move |this, _ev, window, _| {
                                    if this.should_move {
                                        this.should_move = false;
                                        window.start_window_move();
                                    }
                                }))
                                .on_mouse_down_out(cx.listener(move |this, _ev, _window, _cx| {
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
