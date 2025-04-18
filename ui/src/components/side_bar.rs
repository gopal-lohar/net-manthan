use gpui::{IntoElement, MouseButton, Window, div, hsla, prelude::*, rgb};
use ui::{ParentElement, Rems};

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

pub struct SideBar {
    tabs: Vec<Tab>,
    active_tab: Tab,
}

impl SideBar {
    pub fn new() -> Self {
        let tabs = vec![Tab::Home, Tab::Settings, Tab::About];
        Self {
            tabs,
            active_tab: Tab::Home,
        }
    }
}

impl Render for SideBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().text_color(rgb(0xffffff))
            .h_full().flex()
            .child(
                div()
                    .flex()
                    .flex_col().flex_shrink_0()
                    .gap_2()
                    .w_80()
                    .p_8()
                    .pr_0()
                    .h_full()
                    .bg(rgb(0x0D0D16))
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
                        div()
                            .hover(|s| s.bg(rgb(0x101018)))
                            .p_2()
                            .pl_4()
                            .h_11()
                            .rounded_l_md()
                            .child(tab.get_title())
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _ev, _window, _cx| {
                                    this.active_tab = tab_clone;
                                }),
                            )
                    })),
            )
            .child(div().flex_shrink().w_full().p_8().border_5().border_color(rgb(0xff00ff)).child(match self.active_tab {
                Tab::Home => div().child("Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here Home page here Home page hereHome page here Home page here Home page here Home page here Home page here"),
                Tab::Settings => div().child("Settings page here"),
                Tab::About => div().child("About page here"),
            }))
    }
}
