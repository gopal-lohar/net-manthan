use gpui::*;

#[derive(Debug, Clone, Default)]
pub enum WindowBackgroundAppearanceContent {
    Blurred {
        opacity: f32,
    },
    Transparent {
        opacity: f32,
    },
    #[default]
    Opaque,
}

impl From<WindowBackgroundAppearanceContent> for WindowBackgroundAppearance {
    fn from(content: WindowBackgroundAppearanceContent) -> Self {
        match content {
            WindowBackgroundAppearanceContent::Blurred { .. } => {
                WindowBackgroundAppearance::Blurred
            }
            WindowBackgroundAppearanceContent::Transparent { .. } => {
                WindowBackgroundAppearance::Transparent
            }
            WindowBackgroundAppearanceContent::Opaque => WindowBackgroundAppearance::Opaque,
        }
    }
}

impl WindowBackgroundAppearanceContent {
    pub fn opacity(&self) -> f32 {
        match self {
            WindowBackgroundAppearanceContent::Blurred { opacity }
            | WindowBackgroundAppearanceContent::Transparent { opacity } => *opacity,
            WindowBackgroundAppearanceContent::Opaque => 1.0,
        }
    }
}

pub fn get_window_options(app: &mut App) -> WindowOptions {
    let display_id_maybe = app.displays().last().map(|d| d.id());
    let bounds = Bounds::centered(display_id_maybe, size(px(400.0), px(600.0)), app);
    WindowOptions {
        display_id: display_id_maybe,
        focus: true,
        show: true,
        is_movable: true,
        kind: WindowKind::Normal,
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_decorations: Some(gpui::WindowDecorations::Client),
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(Point::new(px(9.0), px(9.0))),
        }),
        ..Default::default()
    }
}

pub fn blur_window(window: &mut Window) {
    window.set_background_appearance(WindowBackgroundAppearance::from(
        WindowBackgroundAppearanceContent::Blurred { opacity: 0.9 },
    ));
}
