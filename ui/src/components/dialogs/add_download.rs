use crate::{
    components::{
        icons::{Icon, themed_icon},
        input::{ValidatedInput, ValidatedInputMessage},
    },
    styles::constants::FONT_SIZE_BODY,
};
use engine::types::{
    messages::DownloadRequestMessage,
    request::{DownloadRequest, Headers},
};
use iced::{
    Alignment, Background, Border, Color, Element,
    Length::{self, Fill},
    Padding, Task, Theme,
    widget::{
        Scrollable, Space, button, column, container, mouse_area, row,
        scrollable::{Direction, Scrollbar},
        stack, text,
    },
};
use rfd::FileDialog;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AddDownload {
    link_input: ValidatedInput,
    file_name_input: ValidatedInput,
    directory_input: ValidatedInput,
}

impl Default for AddDownload {
    fn default() -> Self {
        Self {
            link_input: ValidatedInput::new("Link", "Enter your download link here")
                .non_empty()
                .custom_validation(|value| {
                    if value.trim().is_empty() {
                        return Ok(());
                    }
                    if value.starts_with("http://") || value.starts_with("https://") {
                        Ok(())
                    } else {
                        Err("Must be a valid URL starting with http:// or https://".to_string())
                    }
                }),
            file_name_input: ValidatedInput::new("File Name", "Enter file name here"),
            directory_input: ValidatedInput::new("Directory", "Enter directory path here")
                .non_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AddDownloadMessage {
    LinkInput(ValidatedInputMessage),
    FileNameInput(ValidatedInputMessage),
    DirectoryInput(ValidatedInputMessage),
    OpenDirectoryChooser,
    DirectorySelected(Option<PathBuf>),
    Clear,
    Hide,
    StartWrapper,
    Start(DownloadRequestMessage),
    DoNothing,
}

impl AddDownload {
    pub fn update(&mut self, message: AddDownloadMessage) -> Task<AddDownloadMessage> {
        match message {
            AddDownloadMessage::LinkInput(msg) => {
                self.link_input.update(msg);
                self.link_input.validate();
                Task::none()
            }
            AddDownloadMessage::FileNameInput(msg) => {
                self.file_name_input.update(msg);
                self.file_name_input.validate();
                Task::none()
            }
            AddDownloadMessage::DirectoryInput(msg) => {
                self.directory_input.update(msg);
                self.directory_input.validate();
                Task::none()
            }
            AddDownloadMessage::OpenDirectoryChooser => Task::perform(
                async {
                    FileDialog::new()
                        .set_title("Choose a directory")
                        .pick_folder()
                },
                |dir: Option<PathBuf>| AddDownloadMessage::DirectorySelected(dir),
            ),
            AddDownloadMessage::DirectorySelected(directory) => {
                if let Some(dir) = directory {
                    self.directory_input.value = dir.to_string_lossy().to_string();
                };
                Task::none()
            }
            AddDownloadMessage::Clear => {
                *self = Self::default();
                Task::done(AddDownloadMessage::Hide)
            }
            AddDownloadMessage::StartWrapper => {
                Task::done(AddDownloadMessage::Start(DownloadRequestMessage {
                    request: DownloadRequest {
                        directory: self.directory_input.value.clone(),
                        url: self.link_input.value.clone(),
                        rename: if self.file_name_input.value.is_empty() {
                            None
                        } else {
                            Some(self.file_name_input.value.clone())
                        },
                        headers: Headers::none(),
                    },
                    config: None,
                    info: None,
                }))
            }
            //handled in the grand parent (DownloadManager`)
            AddDownloadMessage::Start(_) => Task::none(),
            // Hide handled in the parent (Dialog)
            AddDownloadMessage::Hide => Task::none(),
            AddDownloadMessage::DoNothing => Task::none(),
        }
    }

    pub fn view(&self) -> Element<AddDownloadMessage> {
        let modal_header = row![
            text("Add a new Download").size(1.25 * FONT_SIZE_BODY),
            Space::with_width(Fill),
            button(themed_icon(Icon::Close, FONT_SIZE_BODY * 1.25, None))
                .on_press(AddDownloadMessage::Hide)
                .style(button::text)
        ]
        .width(Length::Fill)
        .height(FONT_SIZE_BODY * 3.)
        .padding(Padding {
            left: 1.25 * FONT_SIZE_BODY,
            ..Default::default()
        })
        .align_y(Alignment::Center);

        let modal_content = column![
            self.link_input
                .view(FONT_SIZE_BODY, AddDownloadMessage::LinkInput),
            self.file_name_input
                .view(FONT_SIZE_BODY, AddDownloadMessage::FileNameInput),
            row![
                self.directory_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::DirectoryInput),
                container(
                    button(themed_icon(Icon::Directory, 1.5 * FONT_SIZE_BODY, None))
                        .style(button::text)
                        .on_press(AddDownloadMessage::OpenDirectoryChooser)
                        .padding(0.65 * FONT_SIZE_BODY)
                )
                .padding(Padding {
                    top: FONT_SIZE_BODY * (1.5 + 0.25), // line height + spacing
                    ..Default::default()
                })
            ],
            row![
                button(text("Cancel"))
                    .padding(Padding {
                        left: FONT_SIZE_BODY,
                        top: 0.75 * FONT_SIZE_BODY,
                        bottom: 0.75 * FONT_SIZE_BODY,
                        right: FONT_SIZE_BODY,
                    })
                    .style(button::danger)
                    .on_press(AddDownloadMessage::Clear),
                Space::with_width(Length::Fill),
                button(text("Start"))
                    .padding(Padding {
                        left: FONT_SIZE_BODY,
                        top: 0.75 * FONT_SIZE_BODY,
                        bottom: 0.75 * FONT_SIZE_BODY,
                        right: FONT_SIZE_BODY,
                    })
                    .on_press(AddDownloadMessage::StartWrapper),
            ]
        ]
        .spacing(FONT_SIZE_BODY)
        .padding(Padding {
            top: 0.5 * FONT_SIZE_BODY,
            bottom: 1.25 * FONT_SIZE_BODY,
            left: 1.25 * FONT_SIZE_BODY,
            right: 1.25 * FONT_SIZE_BODY,
        });

        let modal_content =
            Scrollable::with_direction(modal_content, Direction::Vertical(Scrollbar::default()));

        let modal = container(column![modal_header, modal_content])
            .style(|theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(theme.palette().background)),
                border: Border::default()
                    .width(1)
                    .color(theme.palette().primary.scale_alpha(0.1)),
                ..Default::default()
            })
            .width(FONT_SIZE_BODY * 32.);

        let modal_container = container(mouse_area(modal).on_press(AddDownloadMessage::DoNothing))
            .center(0)
            .padding(FONT_SIZE_BODY)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let modal_backdrop: Element<AddDownloadMessage> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .center(0)
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(AddDownloadMessage::Hide)
        .into();

        return stack(vec![modal_backdrop, modal_container]).into();
    }
}
