use super::config::UpdateUiConfig;
use crate::components::{
    custom_title_bar::TitleBarMessage, dialogs::DialogMessage, downloads::DownloadsMessage,
    toast::ToastMessage,
};
use iced::Size;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    ConnectAndRefetch,
    DoNothing,
    DownloadsMessage(DownloadsMessage),
    DialogMessage(DialogMessage),
    Navigate(Page),
    OpenDialog,
    Periodic(Duration),
    Refetch,
    Resized(Size),
    TitleBar(TitleBarMessage),
    Toast(ToastMessage),
    UpdateUiConfig(UpdateUiConfig),
}

#[derive(Debug, Clone)]
pub enum Page {
    Downloading,
    Waiting,
    All,
    Actions,
    Settings,
}
