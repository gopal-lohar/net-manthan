use super::config::UpdateUiConfig;
use crate::components::{custom_title_bar::TitleBarMessage, downloads::DownloadsMessage};
use iced::Size;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    DoNothing,
    Resized(Size),
    TitleBar(TitleBarMessage),
    DownloadsMessage(DownloadsMessage),
    UpdateUiConfig(UpdateUiConfig),
    Periodic(Duration),
    Refetch,
    Navigate(Page),
}

#[derive(Debug, Clone)]
pub enum Page {
    Downloading,
    Waiting,
    All,
    Actions,
    Settings,
}
