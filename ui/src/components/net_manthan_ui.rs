use crate::components::{downloads::Downloads, title_bar::TitleBar};

use gpui::{Context, Entity, IntoElement, Window, div, prelude::*, rgb};
use ui::PlatformStyle;

use super::add_download_dialog::AddDownloadDialog;

pub struct NetManthanUi {
    pub title_bar: Entity<TitleBar>,
    pub side_bar: Entity<Downloads>,
    pub platform_style: PlatformStyle,
    pub add_download_dialog: Entity<AddDownloadDialog>,
    pub show_add_download_dialog: bool,
}

impl Render for NetManthanUi {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0x000000))
            .w_full()
            .h_full()
            .when(
                !window.is_fullscreen()
                    && !window.is_maximized()
                    && match self.platform_style {
                        PlatformStyle::Linux => true,
                        _ => false,
                    },
                |d| d.border_1().border_color(rgb(0x30303f)),
            )
            .text_color(rgb(0xffffff))
            .child(self.title_bar.clone())
            .child(self.side_bar.clone())
            .child(
                div()
                    .id("add-download-button")
                    .absolute()
                    .top_10()
                    .right_10()
                    .text_center()
                    .bg(rgb(0x000000))
                    .w_40()
                    .child("Add Download")
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        this.toggle_dialog(cx);
                    })),
            )
            .when(self.show_add_download_dialog, |d| {
                d.child(self.add_download_dialog.clone())
            })
    }
}

impl NetManthanUi {
    pub fn new(_window: &mut Window, cx: &mut Context<NetManthanUi>) -> Self {
        let platform_style = PlatformStyle::platform();
        Self {
            title_bar: cx.new(|_| TitleBar::new()),
            side_bar: cx.new(|cx| Downloads::new(cx)),
            platform_style,
            add_download_dialog: cx.new(|_| AddDownloadDialog::new()),
            show_add_download_dialog: true,
        }
    }

    fn toggle_dialog(&mut self, cx: &mut Context<NetManthanUi>) {
        self.show_add_download_dialog = !self.show_add_download_dialog;
        cx.notify();
    }
}
