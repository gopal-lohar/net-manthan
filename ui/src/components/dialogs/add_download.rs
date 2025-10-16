use crate::{
    components::{
        icons::{Icon, themed_icon},
        input::{ValidatedInput, ValidatedInputMessage},
    },
    styles::constants::FONT_SIZE_BODY,
};
use engine::types::{
    messages::DownloadRequestMessage,
    others::DownloadConfig,
    request::{DownloadRequest, Headers},
};
use iced::{
    Alignment, Background, Border, Color, Element,
    Length::{self, Fill},
    Padding, Task, Theme,
    widget::{
        Column, Scrollable, Space, button, checkbox, column, container, mouse_area, row,
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
    user_agent_input: ValidatedInput,
    authorization_input: ValidatedInput,
    referer_input: ValidatedInput,
    cookie_input: ValidatedInput,
    connections_per_server_input: ValidatedInput,
    retry_count_input: ValidatedInput,
    buffer_size_input: ValidatedInput,
    update_interval_input: ValidatedInput,
    show_advanced_options: bool,
}

impl Default for AddDownload {
    fn default() -> Self {
        let config = DownloadConfig::default();

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
            user_agent_input: ValidatedInput::new(
                "User Agent",
                "Enter the user agent you want to use",
            )
            .with_value("net-manthan/0.1.0"),
            authorization_input: ValidatedInput::new("Authorization", "Enter authorization"),
            referer_input: ValidatedInput::new("Referer", "Enter Referer site"),
            cookie_input: ValidatedInput::new("Cookie", "Enter cookie here"),
            connections_per_server_input: ValidatedInput::new(
                "Split",
                "Number of parts to split the download into (if available)",
            )
            .as_usize(Some(1), Some(64))
            .with_value(config.connections_per_server.to_string()),
            retry_count_input: ValidatedInput::new(
                "Retry count",
                "Enter retry count for download threads",
            )
            .as_usize(Some(0), Some(10))
            .with_value(config.retry_count.to_string()),
            buffer_size_input: ValidatedInput::new(
                "Buffer Size (KB)",
                "Enter buffer size for download threads",
            )
            .as_usize(Some(1), Some(1024))
            .with_value((config.buffer_size / 1024).to_string()),
            update_interval_input: ValidatedInput::new(
                "Update interval",
                "The interval for which the threads update the download manager",
            )
            .as_usize(Some(100), Some(5000))
            .with_value(config.update_interval.to_string()),
            show_advanced_options: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AddDownloadMessage {
    LinkInput(ValidatedInputMessage),
    FileNameInput(ValidatedInputMessage),
    DirectoryInput(ValidatedInputMessage),
    ToggleAdvanceOptions(bool),
    UserAgentInput(ValidatedInputMessage),
    AuthorizationInput(ValidatedInputMessage),
    RefererInput(ValidatedInputMessage),
    CookieInput(ValidatedInputMessage),
    ConnectionsPerServerInput(ValidatedInputMessage),
    RetryCountInput(ValidatedInputMessage),
    BufferSizeInput(ValidatedInputMessage),
    UpdateIntervalInput(ValidatedInputMessage),
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
            AddDownloadMessage::UserAgentInput(msg) => {
                self.user_agent_input.update(msg);
                self.user_agent_input.validate();
                Task::none()
            }
            AddDownloadMessage::AuthorizationInput(msg) => {
                self.authorization_input.update(msg);
                self.authorization_input.validate();
                Task::none()
            }
            AddDownloadMessage::RefererInput(msg) => {
                self.referer_input.update(msg);
                self.referer_input.validate();
                Task::none()
            }
            AddDownloadMessage::CookieInput(msg) => {
                self.cookie_input.update(msg);
                self.cookie_input.validate();
                Task::none()
            }
            AddDownloadMessage::ConnectionsPerServerInput(msg) => {
                self.connections_per_server_input.update(msg);
                self.connections_per_server_input.validate();
                Task::none()
            }
            AddDownloadMessage::RetryCountInput(msg) => {
                self.retry_count_input.update(msg);
                self.retry_count_input.validate();
                Task::none()
            }
            AddDownloadMessage::BufferSizeInput(msg) => {
                self.buffer_size_input.update(msg);
                self.buffer_size_input.validate();
                Task::none()
            }
            AddDownloadMessage::UpdateIntervalInput(msg) => {
                self.update_interval_input.update(msg);
                self.update_interval_input.validate();
                Task::none()
            }
            AddDownloadMessage::Clear => {
                *self = Self::default();
                Task::done(AddDownloadMessage::Hide)
            }
            AddDownloadMessage::StartWrapper => {
                self.link_input.validate();
                self.directory_input.validate();
                self.file_name_input.validate();

                if self.show_advanced_options {
                    self.user_agent_input.validate();
                    self.authorization_input.validate();
                    self.referer_input.validate();
                    self.cookie_input.validate();
                    self.connections_per_server_input.validate();
                    self.retry_count_input.validate();
                    self.buffer_size_input.validate();
                    self.update_interval_input.validate();
                }

                let mut all_valid =
                    self.link_input.is_valid() && self.directory_input.is_valid() && self.file_name_input.is_valid();

                if self.show_advanced_options {
                    all_valid = all_valid
                        && self.user_agent_input.is_valid()
                        && self.authorization_input.is_valid()
                        && self.referer_input.is_valid()
                        && self.cookie_input.is_valid()
                        && self.connections_per_server_input.is_valid()
                        && self.retry_count_input.is_valid()
                        && self.buffer_size_input.is_valid()
                        && self.update_interval_input.is_valid();
                }

                if all_valid {
                    let config = if self.show_advanced_options {
                        Some(DownloadConfig {
                            connections_per_server: self
                                .connections_per_server_input
                                .value
                                .parse()
                                .unwrap_or(8),
                            retry_count: self.retry_count_input.value.parse().unwrap_or(3),
                            buffer_size: self
                                .buffer_size_input
                                .value
                                .parse::<usize>()
                                .unwrap_or(256)
                                * 1024,
                            update_interval: self
                                .update_interval_input
                                .value
                                .parse()
                                .unwrap_or(500),
                        })
                    } else {
                        None
                    };

                    Task::done(AddDownloadMessage::Start(DownloadRequestMessage {
                        request: DownloadRequest {
                            directory: self.directory_input.value.clone(),
                            url: self.link_input.value.clone(),
                            rename: if self.file_name_input.value.is_empty() {
                                None
                            } else {
                                Some(self.file_name_input.value.clone())
                            },
                            headers: Headers {
                                user_agent: (!self.user_agent_input.value.is_empty())
                                    .then_some(self.user_agent_input.value.clone()),
                                authorization: (!self.authorization_input.value.is_empty())
                                    .then_some(self.authorization_input.value.clone()),
                                referer: (!self.referer_input.value.is_empty())
                                    .then_some(self.referer_input.value.clone()),
                                cookie: (!self.cookie_input.value.is_empty())
                                    .then_some(self.cookie_input.value.clone()),
                            },
                        },
                        config,
                        info: None,
                    }))
                } else {
                    Task::none()
                }
            }
            AddDownloadMessage::ToggleAdvanceOptions(visible) => {
                self.show_advanced_options = visible;
                Task::none()
            }
            //handled in the grand parent (DownloadManager`)
            AddDownloadMessage::Start(_) => Task::none(),
            // Hide handled in the parent (Dialog)
            AddDownloadMessage::Hide => Task::none(),
            AddDownloadMessage::DoNothing => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, AddDownloadMessage> {
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

        let mut modal_content = vec![
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
            ]
            .into(),
            container(
                checkbox("Show Advanced Options", self.show_advanced_options)
                    .on_toggle(AddDownloadMessage::ToggleAdvanceOptions),
            )
            .into(),
        ];

        if self.show_advanced_options {
            modal_content.push(
                self.user_agent_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::UserAgentInput)
                    .into(),
            );
            modal_content.push(
                self.authorization_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::AuthorizationInput)
                    .into(),
            );
            modal_content.push(
                self.referer_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::RefererInput)
                    .into(),
            );
            modal_content.push(
                self.cookie_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::CookieInput)
                    .into(),
            );
            modal_content.push(
                self.connections_per_server_input
                    .view(
                        FONT_SIZE_BODY,
                        AddDownloadMessage::ConnectionsPerServerInput,
                    )
                    .into(),
            );
            modal_content.push(
                self.retry_count_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::RetryCountInput)
                    .into(),
            );

            modal_content.push(
                self.buffer_size_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::BufferSizeInput)
                    .into(),
            );

            modal_content.push(
                self.update_interval_input
                    .view(FONT_SIZE_BODY, AddDownloadMessage::UpdateIntervalInput)
                    .into(),
            );
        }

        modal_content.push(
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
            .into(),
        );

        let modal_content = Scrollable::with_direction(
            Column::from_vec(modal_content)
                .spacing(FONT_SIZE_BODY)
                .padding(Padding {
                    top: 0.5 * FONT_SIZE_BODY,
                    bottom: 1.25 * FONT_SIZE_BODY,
                    left: 1.25 * FONT_SIZE_BODY,
                    right: 1.25 * FONT_SIZE_BODY,
                })
                .width(Length::Fill),
            Direction::Vertical(Scrollbar::default()),
        );

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
