use crate::helpers::theme::Theme;

use super::home::Home;
use gpui::{Entity, IntoElement, Window, div, hsla, prelude::*, rgb};
use ui::{ParentElement, Pixels, Rems, SharedString};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Home,
    Settings,
    About,
}

impl Tab {
    fn get_title(&self) -> String {
        match self {
            Tab::Home => "Home".to_string(),
            Tab::Settings => "Settings".to_string(),
            Tab::About => "About".to_string(),
        }
    }
}

pub struct Downloads {
    tabs: Vec<Tab>,
    active_tab: Tab,
    home: Entity<Home>,
}

impl Downloads {
    pub fn new(cx: &mut Context<Downloads>) -> Self {
        let tabs = vec![Tab::Home, Tab::Settings, Tab::About];
        Self {
            tabs,
            active_tab: Tab::Home,
            home: cx.new(|cx| Home::new(cx)),
        }
    }
}

impl Render for Downloads {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        div()
            .text_color(rgb(0xffffff))
            .h_full()
            .flex()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .gap_2()
                    .w_80()
                    .p_8()
                    .pr_0()
                    .h_full()
                    .bg(theme.base_blur)
                    .child(
                        div()
                            .text_color(hsla(0.0, 0.0, 0.0, 0.0))
                            .p_2()
                            .pl_4()
                            .w_72()
                            .h_11()
                            .bg(rgb(0x000000))
                            .rounded_l_md()
                            .absolute()
                            .top(Rems(
                                2.0 + (self
                                    .tabs
                                    .iter()
                                    .position(|x| x == &self.active_tab)
                                    .unwrap_or(0)) as f32
                                    * 3.25,
                            ))
                            .child("Locator"),
                    )
                    .children(self.tabs.iter().map(|tab| {
                        let tab_clone = tab.clone();
                        let id_str = format!("tab-switch-{}", tab_clone.get_title());
                        let ssid = SharedString::from(id_str);
                        div()
                            .id(ssid)
                            .hover(|s| s.bg(rgb(0x101018)))
                            .p_2()
                            .pl_4()
                            .h_11()
                            .rounded_l_md()
                            .child(tab.get_title())
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _ev, _window, _cx| {
                                this.active_tab = tab_clone;
                            }))
                    })),
            )
            .child(
                div()
                    .flex_shrink()
                    .w(window.viewport_size().width - Pixels(320.0 + 2.0))
                    .p_8()
                    .child(match self.active_tab {
                        Tab::Home => div().child(self.home.clone()),
                        Tab::Settings => div().child("Settings page here"),
                        Tab::About => div().child("About page here"),
                    }),
            )
    }
}
