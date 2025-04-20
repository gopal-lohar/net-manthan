use std::path::Path;

use gpui::{ClipboardItem, IntoElement, PromptLevel, Window, div, prelude::*, rgb};
use ui::{DefiniteLength, ParentElement, SharedString};

struct Download {}

pub struct Home {
    downloads: Vec<Download>,
}

impl Home {
    pub fn new() -> Self {
        Self {
            downloads: Vec::new(),
        }
    }
}

impl Render for Home {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Downloads...")
    }
}
