use super::icons::{Icon, themed_icon};
use crate::styles::constants::{BORDER_ROUNDED_RADIUS, FONT_SIZE_BODY};
use engine::{
    helpers::format::{format_bytes, format_duration, format_speed},
    types::{download::Download, status::DownloadStatus},
};
use iced::{
    Alignment, Background, Border, Element, Length, Padding, Task, Theme,
    border::Radius,
    widget::{
        Column, Scrollable, Space, button, column, container, row,
        scrollable::{Direction, Scrollbar},
        text,
    },
};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct Downloads {
    all: Result<Vec<Download>, String>,
}

impl Default for Downloads {
    fn default() -> Self {
        Self {
            all: Err("not fetched yet".into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DownloadsMessage {
    AddDownload,
    ShowAddDownloadModal,
    UpdateDownloads(Result<Vec<Download>, String>),
    ShowDetails(i64),
    PauseDownload(i64),
    ResumeDownload(i64),
}

impl Downloads {
    pub fn update(&mut self, message: DownloadsMessage) -> Task<DownloadsMessage> {
        match message {
            DownloadsMessage::UpdateDownloads(downloads) => {
                self.all = downloads;
                Task::none()
            }
            _ => {
                warn!(
                    "DownloadMessage recieved but not implemented yet. message: {:?}",
                    message
                );
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, DownloadsMessage> {
        let mut downloads_column = Vec::new();

        let downloads_column: Element<DownloadsMessage> = match &self.all {
            Ok(downloads) => {
                if downloads.is_empty() {
                    Column::new()
                        .push(text("No downloads").center().width(Length::Fill))
                        .width(Length::Fill)
                        .padding(FONT_SIZE_BODY)
                        .into()
                } else {
                    for download in downloads {
                        downloads_column.push(download_view(&download));
                    }
                    Column::from_vec(downloads_column)
                        .width(Length::Fill)
                        .padding(FONT_SIZE_BODY)
                        .spacing(FONT_SIZE_BODY)
                        .into()
                }
            }
            Err(error) => Column::new()
                .push(
                    text(format!("Error: {}", error))
                        .center()
                        .width(Length::Fill)
                        .style(text::danger),
                )
                .padding(FONT_SIZE_BODY)
                .into(),
        };

        let downloads: Element<DownloadsMessage> = Scrollable::with_direction(
            downloads_column,
            Direction::Vertical(
                Scrollbar::default()
                    .width(0.5 * FONT_SIZE_BODY)
                    .scroller_width(0.5 * FONT_SIZE_BODY)
                    .margin(0.),
            ),
        )
        .into();
        column![
            row![
                container("Home").width(Length::Fill),
                button(
                    row![
                        themed_icon(Icon::Add, FONT_SIZE_BODY * 1.25, None),
                        text("Add Download")
                    ]
                    .spacing(FONT_SIZE_BODY * 0.5)
                    .align_y(Alignment::Center)
                )
                .style(button::text)
                .on_press(DownloadsMessage::ShowAddDownloadModal),
            ]
            .padding(
                Padding::new(FONT_SIZE_BODY * 0.5)
                    .left(FONT_SIZE_BODY)
                    .right(FONT_SIZE_BODY)
            )
            .align_y(Alignment::Center),
            downloads
        ]
        .padding(FONT_SIZE_BODY)
        .into()
    }
}

pub fn download_view(download: &Download) -> Element<'_, DownloadsMessage> {
    let progress = match download.progress_percentage() {
        Some(p) => (p * 10.).round() as u16,
        None => 0,
    };
    let downloading = download.get_status() == DownloadStatus::Downloading;
    let unknown_filename: String = "Filename Unknown".into();
    container(
        column![
            column![
                row![
                    text(download.get_filename().unwrap_or(unknown_filename))
                        .size(FONT_SIZE_BODY)
                        .width(Length::Fill),
                    container(
                        button(themed_icon(
                            if downloading { Icon::Pause } else { Icon::Play },
                            1.5 * FONT_SIZE_BODY,
                            None
                        ))
                        .on_press(if downloading {
                            DownloadsMessage::PauseDownload(download.id)
                        } else {
                            DownloadsMessage::ResumeDownload(download.id)
                        })
                        .style(move |theme, status| {
                            iced::widget::button::Style {
                                border: Border {
                                    radius: Radius::new(BORDER_ROUNDED_RADIUS),
                                    ..Default::default()
                                },
                                ..button::text(theme, status)
                            }
                        })
                        .padding(0.25 * FONT_SIZE_BODY)
                    ),
                    container(
                        button(themed_icon(Icon::Info, 1.5 * FONT_SIZE_BODY, None))
                            .style(move |theme, status| {
                                iced::widget::button::Style {
                                    border: Border {
                                        radius: Radius::new(FONT_SIZE_BODY),
                                        ..Default::default()
                                    },
                                    ..button::text(theme, status)
                                }
                            })
                            .padding(0.25 * FONT_SIZE_BODY)
                            .on_press(DownloadsMessage::ShowDetails(download.id.clone()))
                    )
                ]
                .align_y(Alignment::Center)
                .spacing(0.5 * FONT_SIZE_BODY),
                row![
                    text(format!(
                        "{:.1}% | {} / {}",
                        download.progress_percentage().unwrap_or(0.),
                        format_bytes(download.bytes_downloaded()),
                        if let Some(size) = download.total_size() {
                            format_bytes(size)
                        } else {
                            "Unknown".into()
                        }
                    )),
                    Space::new(Length::Fill, 0),
                    text(format!(
                        "{} | {} / {}",
                        format_speed(download.total_speed()),
                        format_duration(download.time_stamps.active_time),
                        match download.estimated_time_remaining() {
                            Some(t) => format_duration(t),
                            None => "Unknown".into(),
                        },
                    ))
                ]
                .spacing(FONT_SIZE_BODY)
            ],
            container(row![
                if progress > 0 {
                    container(Space::with_height(Length::Fill))
                        .width(Length::FillPortion(progress))
                        .style(move |theme: &Theme| iced::widget::container::Style {
                            background: Some(Background::from(theme.palette().primary)),
                            border: Border {
                                radius: Radius {
                                    top_left: FONT_SIZE_BODY * 0.25,
                                    bottom_left: FONT_SIZE_BODY * 0.25,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                } else {
                    container(Space::with_height(Length::Fill)).width(Length::Fixed(0.0))
                },
                if progress < 1000 {
                    container(Space::with_height(Length::Fill))
                        .width(Length::FillPortion(1000 - progress))
                } else {
                    container(Space::with_height(Length::Fill)).width(Length::Fixed(0.0))
                },
            ])
            .width(Length::Fill)
            .height(FONT_SIZE_BODY * 0.25)
            .style(move |theme: &Theme| {
                iced::widget::container::Style {
                    background: Some(Background::from(
                        theme
                            .extended_palette()
                            .background
                            .weak
                            .color
                            .scale_alpha(0.1),
                    )),
                    border: Border {
                        radius: Radius::new(FONT_SIZE_BODY * 0.25),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        ]
        .spacing(FONT_SIZE_BODY * 0.75),
    )
    .style(move |theme: &Theme| iced::widget::container::Style {
        border: Border {
            width: 1.,
            color: theme
                .extended_palette()
                .background
                .weak
                .color
                .scale_alpha(0.1),
            radius: Radius::new(FONT_SIZE_BODY * 0.25),
        },
        ..Default::default()
    })
    .padding(FONT_SIZE_BODY)
    .into()
}
