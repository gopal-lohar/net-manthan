use add_download::{AddDownload, AddDownloadMessage};
use iced::{Task, widget::Space};

pub mod add_download;

pub enum Dialog {
    AddDownload(AddDownload),
    None,
}

#[derive(Debug, Clone)]
pub enum DialogMessage {
    AddDownload(AddDownloadMessage),
    DoNothing,
}

impl Dialog {
    pub fn update(&mut self, message: DialogMessage) -> Task<DialogMessage> {
        match message {
            DialogMessage::AddDownload(message) => match self {
                Dialog::AddDownload(add_download) => match message {
                    AddDownloadMessage::Hide => {
                        *self = Dialog::None;
                        Task::none()
                    }
                    m => add_download.update(m).map(DialogMessage::AddDownload),
                },
                _ => Task::none(),
            },
            DialogMessage::DoNothing => Task::none(),
        }
    }

    pub fn view(&self) -> iced::Element<DialogMessage> {
        match self {
            Dialog::AddDownload(add_download) => {
                add_download.view().map(DialogMessage::AddDownload)
            }
            Dialog::None => Space::new(0, 0).into(),
        }
    }
}
