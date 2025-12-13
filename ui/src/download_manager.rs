use crate::{
    components::{
        custom_title_bar::{TitleBar, TitleBarMessage},
        dialogs::{
            Dialog, DialogMessage,
            add_download::{AddDownload, AddDownloadMessage},
        },
        downloads::{Downloads, DownloadsMessage},
        icons::{Icon, themed_icon},
        toast::{Toast, ToastMessage, ToastVariant},
    },
    styles::constants::{BORDER_WIDTH, FONT_SIZE_BODY},
    types::{
        config::{Config, UpdateUiConfig},
        message::{Message, Page},
        theme::ThemeOptions,
    },
};
use iced::{
    Alignment, Background, Element, Length, Padding, Pixels, Subscription, Task, Theme,
    widget::{
        PickList, Space,
        button::{self, Button, Status, Style},
        column, container, row, slider, stack, text, toggler,
    },
    window,
};
use std::{sync::Arc, time::Duration};
use tracing::trace;
use utils::rpc::{client::Client, messages::RpcRequest};

pub struct DownloadManager {
    client: Arc<Client>,
    config: Config,
    current_page: Page,
    default_page: Page,
    dialog: Dialog,
    downloads: Downloads,
    title_bar: TitleBar,
    toast: Toast,
}

impl DownloadManager {
    pub fn new(client: Arc<Client>, config: Config) -> Self {
        Self {
            config,
            client,
            title_bar: TitleBar::default(),
            current_page: Page::AllDownloads,
            default_page: Page::AllDownloads,
            downloads: Downloads::default(),
            toast: Toast::default(),
            dialog: Dialog::None,
        }
    }
}

impl DownloadManager {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(page) => {
                self.current_page = page;
                Task::none()
            }
            Message::DoNothing => Task::none(),
            Message::UpdateUiConfig(message) => {
                let task = self.config.ui.update(message).map(Message::UpdateUiConfig);
                let config = self.config.clone();
                let config_path = self.config.ui.config_path.clone();
                tokio::spawn(async move {
                    let config_str = toml::to_string_pretty(&config).unwrap();
                    std::fs::write(config_path, config_str).unwrap();
                });
                task
            }
            Message::Toast(message) => self.toast.update(message).map(Message::Toast),
            Message::Periodic(_) => Task::done(Message::Refetch),
            Message::ConnectAndRefetch => {
                trace!("connecting");
                let client = Arc::clone(&self.client);
                Task::perform(
                    async move {
                        let _ = client.connect().await;
                        client.get_downloads().await
                    },
                    |response| {
                        if let Err(_) = response {
                            DownloadsMessage::UpdateDownloads(response)
                        } else {
                            DownloadsMessage::UpdateDownloads(response)
                        }
                    },
                )
                .map(Message::DownloadsMessage)
            }
            Message::Refetch => {
                trace!("Refetching the downloads");
                let client = Arc::clone(&self.client);
                Task::perform(async move { client.get_downloads().await }, |response| {
                    if let Err(_) = response {
                        Message::ConnectAndRefetch
                    } else {
                        Message::DownloadsMessage(DownloadsMessage::UpdateDownloads(response))
                    }
                })
            }
            Message::DownloadsMessage(message) => match message {
                DownloadsMessage::ShowAddDownloadModal => Task::done(Message::OpenDialog),
                DownloadsMessage::ResumeDownload(id) => {
                    let client = Arc::clone(&self.client);
                    Task::perform(
                        async move { client.send(RpcRequest::ResumeDownload(id)).await },
                        move |response| match response {
                            Ok(message) => match message {
                                utils::rpc::messages::RpcResponse::Success => {
                                    Message::Toast(ToastMessage::Show(
                                        format!("Download resumed: {id}"),
                                        ToastVariant::Info,
                                    ))
                                }
                                _ => Message::Toast(ToastMessage::Show(
                                    format!("Could not resume the Download: {id}"),
                                    ToastVariant::Error,
                                )),
                            },
                            Err(_) => Message::Toast(ToastMessage::Show(
                                format!(
                                    "Something went wrong. Could not resume the Download: {id}"
                                ),
                                ToastVariant::Error,
                            )),
                        },
                    )
                }
                DownloadsMessage::PauseDownload(id) => {
                    let client = Arc::clone(&self.client);
                    Task::perform(
                        async move { client.send(RpcRequest::PauseDownload(id)).await },
                        move |response| match response {
                            Ok(message) => match message {
                                utils::rpc::messages::RpcResponse::Success => {
                                    Message::Toast(ToastMessage::Show(
                                        format!("Download paused: {id}"),
                                        ToastVariant::Info,
                                    ))
                                }
                                _ => Message::Toast(ToastMessage::Show(
                                    format!("Could not pause the Download: {id}"),
                                    ToastVariant::Error,
                                )),
                            },
                            Err(_) => Message::Toast(ToastMessage::Show(
                                format!("Something went wrong. Could not pause the Download: {id}"),
                                ToastVariant::Error,
                            )),
                        },
                    )
                }
                message => self
                    .downloads
                    .update(message)
                    .map(Message::DownloadsMessage),
            },
            Message::DialogMessage(message) => match message {
                DialogMessage::AddDownload(message) => match message {
                    AddDownloadMessage::Start(request) => {
                        let client = Arc::clone(&self.client);
                        Task::perform(
                            async move {
                                let _ = client.send(RpcRequest::DownloadRequest(request)).await;
                                ()
                            },
                            |_| {
                                Message::DialogMessage(DialogMessage::AddDownload(
                                    AddDownloadMessage::Hide,
                                ))
                            },
                        )
                    }
                    _ => self
                        .dialog
                        .update(DialogMessage::AddDownload(message))
                        .map(Message::DialogMessage),
                },
                message => self.dialog.update(message).map(Message::DialogMessage),
            },
            Message::OpenDialog => {
                self.dialog = Dialog::AddDownload(AddDownload::default());
                Task::none()
            }
            Message::Resized(size) => self
                .config
                .ui
                .update(UpdateUiConfig::Size(size))
                .map(Message::UpdateUiConfig),
            Message::TitleBar(message) => self.title_bar.update(message).map(Message::TitleBar),
        }
    }

    pub fn settings_view(&self) -> Element<'_, Message> {
        let theme_picker: Element<UpdateUiConfig> = container(
            PickList::new(
                vec![
                    ThemeOptions::Light,
                    ThemeOptions::Dark,
                    ThemeOptions::Dracula,
                    ThemeOptions::Nord,
                    ThemeOptions::SolarizedDark,
                    ThemeOptions::SolarizedLight,
                    ThemeOptions::GruvboxDark,
                    ThemeOptions::GruvboxLight,
                    ThemeOptions::CatppuccinLatte,
                    ThemeOptions::CatppuccinFrappe,
                    ThemeOptions::CatppuccinMacchiato,
                    ThemeOptions::CatppuccinMocha,
                    ThemeOptions::TokyoNight,
                    ThemeOptions::TokyoNightStorm,
                    ThemeOptions::TokyoNightLight,
                    ThemeOptions::KanagawaWave,
                    ThemeOptions::KanagawaDragon,
                    ThemeOptions::KanagawaLotus,
                    ThemeOptions::Moonfly,
                    ThemeOptions::Nightfly,
                    ThemeOptions::Oxocarbon,
                    ThemeOptions::Ferra,
                ],
                Some(self.config.ui.theme),
                UpdateUiConfig::Theme,
            )
            .placeholder("Choose a theme..."),
        )
        .into();

        let interface_scale: Element<UpdateUiConfig> = slider(
            0.33..=3.,
            self.config.ui.scale_factor,
            UpdateUiConfig::ScaleFactor,
        )
        .step(0.01)
        .width(FONT_SIZE_BODY * 20. / (self.config.ui.scale_factor as f32))
        .into();

        let custom_title_bar_toggle: Element<UpdateUiConfig> =
            toggler(self.config.ui.custom_decoration)
                .size(FONT_SIZE_BODY * 1.5)
                .on_toggle(UpdateUiConfig::CustomDecoration)
                .into();

        column![
            row![
                Button::new(themed_icon(Icon::ArrowBack, FONT_SIZE_BODY * 1.25, None))
                    .style(button::text)
                    .on_press(Message::Navigate(self.default_page.clone())),
                text("Settings"),
            ]
            .padding(FONT_SIZE_BODY * 0.5)
            .align_y(Alignment::Center),
            column![
                row![text("Theme: "), theme_picker.map(Message::UpdateUiConfig)]
                    .padding(FONT_SIZE_BODY)
                    .align_y(Alignment::Center)
                    .spacing(FONT_SIZE_BODY),
                row![
                    text("Interface Scale: "),
                    text(format!("{:.0}%", self.config.ui.scale_factor * 100.)),
                    Space::new(Length::Fill, 0.),
                    interface_scale.map(Message::UpdateUiConfig)
                ]
                .padding(FONT_SIZE_BODY)
                .align_y(Alignment::Center)
                .spacing(FONT_SIZE_BODY),
                row![
                    text("Themed Titlebar: "),
                    custom_title_bar_toggle.map(Message::UpdateUiConfig)
                ]
                .padding(FONT_SIZE_BODY)
                .align_y(Alignment::Center)
                .spacing(FONT_SIZE_BODY)
            ]
        ]
        .padding(FONT_SIZE_BODY)
        .into()
    }

    fn side_bar_view(&self, collapsed: bool) -> Element<'_, Message> {
        let sidebar_button = |page: &Page, current_page: &Page| {
            let current_page = page == current_page;
            let content = row![themed_icon(
                match page {
                    Page::AllDownloads => Icon::Directory,
                    Page::Downloading => Icon::Downloading,
                    Page::Paused => Icon::Pause,
                    Page::Settings => Icon::Settings,
                },
                FONT_SIZE_BODY * 1.25,
                None
            )];
            let button = Button::new(
                if collapsed {
                    content
                } else {
                    content.push(text(page.as_str().to_owned()))
                }
                .spacing(FONT_SIZE_BODY * 0.5)
                .align_y(Alignment::Center),
            )
            .padding(
                Padding::new(FONT_SIZE_BODY * 0.5)
                    .left(if collapsed {
                        FONT_SIZE_BODY * 1.25
                    } else {
                        FONT_SIZE_BODY * 0.75
                    })
                    .right(FONT_SIZE_BODY * 1.25),
            )
            .on_press(Message::Navigate(page.to_owned()))
            .style(move |theme: &Theme, status: Status| -> Style {
                let palette = theme.extended_palette();

                let base = Style {
                    text_color: palette.background.base.text,
                    background: if current_page {
                        Some(Background::Color(
                            theme.palette().text.scale_alpha(if palette.is_dark {
                                0.02
                            } else {
                                0.2
                            }),
                        ))
                    } else {
                        None
                    },
                    ..Style::default()
                };

                match status {
                    Status::Active | Status::Pressed => base,
                    Status::Hovered => Style {
                        text_color: palette.background.base.text.scale_alpha(0.9),
                        ..base
                    },
                    Status::Disabled => Style {
                        text_color: palette.background.base.text.scale_alpha(0.6),
                        ..base
                    },
                }
            });
            if collapsed {
                button
            } else {
                button.width(FONT_SIZE_BODY * 12.)
            }
        };

        let separator = container("")
            .height(Length::Fill)
            .width(Pixels::from(BORDER_WIDTH))
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(
                        theme
                            .palette()
                            .text
                            .scale_alpha(if palette.is_dark { 0.02 } else { 0.2 })
                            .into(),
                    ),
                    ..Default::default()
                }
            });

        return container(row![
            column![
                sidebar_button(&Page::AllDownloads, &self.current_page),
                sidebar_button(&Page::Downloading, &self.current_page),
                sidebar_button(&Page::Paused, &self.current_page),
                Space::new(0., Length::Fill),
                sidebar_button(&Page::Settings, &self.current_page),
            ]
            .padding(
                Padding::new(0.)
                    .top(FONT_SIZE_BODY * 1.25)
                    .bottom(FONT_SIZE_BODY * 1.25),
            ),
            separator
        ])
        .height(Length::Fill)
        .into();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title_bar = self.title_bar.view(&self.config.ui).map(Message::TitleBar);
        let content = row![
            self.side_bar_view(Into::<iced::Size>::into(self.config.ui.size).width < 1100.),
            match self.current_page {
                Page::Downloading | Page::Paused | Page::AllDownloads => self
                    .downloads
                    .view(&self.current_page)
                    .map(Message::DownloadsMessage),
                Page::Settings => self.settings_view(),
            }
        ];
        let dialog = self.dialog.view().map(Message::DialogMessage);
        let toast = self.toast.view().map(Message::Toast);
        column![title_bar, stack![content, dialog, toast]].into()
    }

    pub fn window_subscription() -> Subscription<Message> {
        // NOTE: Close request can be handled here but not needed now
        iced::event::listen_with(|event, _, _| match event {
            iced::Event::Window(window::Event::Focused) => {
                Some(Message::TitleBar(TitleBarMessage::Focused))
            }
            iced::Event::Window(window::Event::Unfocused) => {
                Some(Message::TitleBar(TitleBarMessage::UnFocused))
            }
            iced::Event::Window(window::Event::Resized(s)) => Some(Message::Resized(s)),
            _ => None,
        })
    }

    fn time_subscription() -> Subscription<Message> {
        let interval = Duration::from_millis(1000);
        iced::time::every(interval).map(|_| Message::Periodic(Duration::from_micros(1000)))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([Self::window_subscription(), Self::time_subscription()])
    }

    pub fn theme(&self) -> Theme {
        self.config.ui.theme.into()
    }

    pub fn scale_factor(&self) -> f64 {
        self.config.ui.scale_factor
    }
}
