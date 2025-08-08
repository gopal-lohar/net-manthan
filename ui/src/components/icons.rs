use iced::{Color, Theme, widget::svg};

pub const ADD_ICON: &[u8] = include_bytes!("../../assets/icons/add.svg");
pub const ARROW_BACK_ICON: &[u8] = include_bytes!("../../assets/icons/arrow_back.svg");
pub const CHECK_ICON: &[u8] = include_bytes!("../../assets/icons/check.svg");
pub const CLOSE_ICON: &[u8] = include_bytes!("../../assets/icons/close.svg");
pub const DELETE_ICON: &[u8] = include_bytes!("../../assets/icons/delete.svg");
pub const DIRECTORY_ICON: &[u8] = include_bytes!("../../assets/icons/directory.svg");
pub const DOWNLOADING_ICON: &[u8] = include_bytes!("../../assets/icons/downloading.svg");
pub const ERROR_ICON: &[u8] = include_bytes!("../../assets/icons/error.svg");
pub const INFO_ICON: &[u8] = include_bytes!("../../assets/icons/info_i.svg");
pub const MINIMIZE_ICON: &[u8] = include_bytes!("../../assets/icons/minimize.svg");
pub const MAXIMIZE_ICON: &[u8] = include_bytes!("../../assets/icons/maximize.svg");
pub const PAUSE_ICON: &[u8] = include_bytes!("../../assets/icons/pause.svg");
pub const PLAY_ICON: &[u8] = include_bytes!("../../assets/icons/play.svg");
pub const RESTORE_ICON: &[u8] = include_bytes!("../../assets/icons/restore.svg");
pub const WARNING_ICON: &[u8] = include_bytes!("../../assets/icons/warning.svg");
pub const SETTINGS_ICON: &[u8] = include_bytes!("../../assets/icons/settings.svg");

pub enum Icon {
    Add,
    ArrowBack,
    Check,
    Close,
    Delete,
    Directory,
    Downloading,
    Error,
    Info,
    Minimize,
    Maximize,
    Pause,
    Play,
    Restore,
    Warning,
    Settings,
}

pub fn themed_icon<'a, T>(icon: Icon, size: f32, color: Option<Color>) -> iced::Element<'a, T> {
    svg(svg::Handle::from_memory(match icon {
        Icon::Add => ADD_ICON,
        Icon::ArrowBack => ARROW_BACK_ICON,
        Icon::Check => CHECK_ICON,
        Icon::Close => CLOSE_ICON,
        Icon::Delete => DELETE_ICON,
        Icon::Directory => DIRECTORY_ICON,
        Icon::Downloading => DOWNLOADING_ICON,
        Icon::Error => ERROR_ICON,
        Icon::Info => INFO_ICON,
        Icon::Minimize => MINIMIZE_ICON,
        Icon::Maximize => MAXIMIZE_ICON,
        Icon::Pause => PAUSE_ICON,
        Icon::Play => PLAY_ICON,
        Icon::Restore => RESTORE_ICON,
        Icon::Settings => SETTINGS_ICON,
        Icon::Warning => WARNING_ICON,
    }))
    .width(size)
    .height(size)
    .style(move |x: &Theme, _| svg::Style {
        color: Some(color.unwrap_or(x.palette().text)),
    })
    .into()
}
