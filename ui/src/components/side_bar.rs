use gpui::{IntoElement, MouseButton, Window, div, rgb};
use ui::ParentElement;

#[derive(Clone, Copy)]
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
        div()
            .children(self.tabs.iter().map(|tab| {
                let tab_clone = tab.clone();
                div()
                    .child(tab.get_title())
                    .hover(|s| s.bg(rgb(0x101020)))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _window, _cx| {
                            this.active_tab = tab_clone;
                        }),
                    )
            }))
            .child(match self.active_tab {
                Tab::Home => div().child("Home page here"),
                Tab::Settings => div().child("Settings page here"),
                Tab::About => div().child("About page here"),
            })
    }
}
