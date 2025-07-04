// Mostly lifted from https://github.com/huacnlee/gpui-component/blob/main/crates/ui/src/icon.rs
use crate::helpers::theme::Theme;
use gpui::AppContext;
use gpui::{
    AnyElement, App, Entity, Hsla, IntoElement, Pixels, Render, RenderOnce, SharedString,
    StyleRefinement, Styled, Svg, Window, prelude::FluentBuilder as _, svg,
};

/// A size for elements.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    Size(Pixels),
    XSmall,
    Small,
    Medium,
    Large,
}

impl From<Pixels> for Size {
    fn from(size: Pixels) -> Self {
        Size::Size(size)
    }
}

#[derive(IntoElement, Clone)]
pub enum IconName {
    Archive,
    Delete,
    Plus,
    Trash,
    Minimize,
    Restore,
    Maximize,
    Close,
}

impl IconName {
    pub fn path(self) -> SharedString {
        match self {
            IconName::Archive => "icons/archive.svg",
            IconName::Delete => "icons/delete.svg",
            IconName::Plus => "icons/plus.svg",
            IconName::Trash => "icons/trash.svg",
            IconName::Minimize => "icons/generic_minimize.svg",
            IconName::Restore => "icons/generic_restore.svg",
            IconName::Maximize => "icons/generic_maximize.svg",
            IconName::Close => "icons/generic_close.svg",
        }
        .into()
    }

    /// Return the icon as a View<Icon>
    pub fn view(self, cx: &mut App) -> Entity<Icon> {
        Icon::build(self).view(cx)
    }
}

impl From<IconName> for Icon {
    fn from(val: IconName) -> Self {
        Icon::build(val)
    }
}

impl From<IconName> for AnyElement {
    fn from(val: IconName) -> Self {
        Icon::build(val).into_any_element()
    }
}

impl RenderOnce for IconName {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        Icon::build(self)
    }
}

#[derive(IntoElement)]
pub struct Icon {
    base: Svg,
    path: SharedString,
    text_color: Option<Hsla>,
    size: Size,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            base: svg().flex_none().size_4(),
            path: "".into(),
            text_color: None,
            size: Size::Medium,
        }
    }
}

impl Clone for Icon {
    fn clone(&self) -> Self {
        Self::default().path(self.path.clone()).size(self.size)
    }
}

impl Icon {
    pub fn new(icon: impl Into<Icon>) -> Self {
        icon.into()
    }

    fn build(name: IconName) -> Self {
        Self::default().path(name.path())
    }

    pub fn view(self, cx: &mut App) -> Entity<Icon> {
        cx.new(|_| self)
    }

    /// Set the icon path of the Assets bundle
    ///
    /// For example: `icons/foo.svg`
    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = path.into();
        self
    }

    /// Set the size of the icon, default is `IconSize::Medium`
    ///
    /// Also can receive a `ButtonSize` to convert to `IconSize`,
    /// Or a `Pixels` to set a custom size: `px(30.)`
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();

        self
    }

    pub fn transform(mut self, transformation: gpui::Transformation) -> Self {
        self.base = self.base.with_transformation(transformation);
        self
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }

    fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut Window, app: &mut App) -> impl IntoElement {
        let theme = app.global::<Theme>();
        let text_color = self.text_color.unwrap_or_else(|| theme.text.into());

        self.base
            .text_color(text_color)
            .map(|this| match self.size {
                Size::Size(px) => this.size(px),
                Size::XSmall => this.size_3(),
                Size::Small => this.size_3p5(),
                Size::Medium => this.size_4(),
                Size::Large => this.size_6(),
            })
            .path(self.path)
    }
}

impl From<Icon> for AnyElement {
    fn from(val: Icon) -> Self {
        val.into_any_element()
    }
}

impl Render for Icon {
    fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let text_color = self.text_color.unwrap_or_else(|| theme.text.into());

        svg()
            .flex_none()
            .size_4()
            .text_color(text_color)
            .map(|this| match self.size {
                Size::Size(px) => this.size(px),
                Size::XSmall => this.size_3(),
                Size::Small => this.size_3p5(),
                Size::Medium => this.size_4(),
                Size::Large => this.size_6(),
            })
            .path(self.path.clone())
    }
}
