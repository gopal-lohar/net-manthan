use crate::{
    components::{
        custom_title_bar::{TitleBar, TitleBarMessage},
        downloads::{Downloads, DownloadsMessage},
        icons::{Icon, themed_icon},
    },
    styles::constants::FONT_SIZE_BODY,
    types::{
        config::{Config, UpdateUiConfig},
        message::{Message, Page},
        theme::ThemeOptions,
    },
};
use iced::{
    Alignment, Element, Length, Padding, Subscription, Task, Theme,
    widget::{PickList, Space, button, column, container, row, slider, text, toggler},
    window,
};
use std::{sync::Arc, time::Duration};
use tracing::{trace, warn};
use utils::rpc::{
    client::Client,
    messages::{RpcRequest, RpcResponse},
};

pub struct DownloadManager {
    config: Config,
    title_bar: TitleBar,
    current_page: Page,
    downloads: Downloads,
    client: Arc<Client>,
}

impl DownloadManager {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            config: Config::default(),
            client,
            title_bar: TitleBar::default(),
            current_page: Page::Downloading,
            downloads: Downloads::default(),
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
                self.config.ui.update(message).map(Message::UpdateUiConfig)
            }
            Message::Periodic(_) => Task::done(Message::Refetch),
            Message::Refetch => {
                trace!("Refetching the downloads");
                let client = Arc::clone(&self.client);
                Task::perform(
                    async move {
                        match client.send(RpcRequest::GetDownloads(vec![])).await {
                            Ok(res) => match res {
                                RpcResponse::Downloads(d) => Ok(d),
                                _ => {
                                    let err = "Invalid Response to ipc request";
                                    warn!("{}", err);
                                    Err(err.to_string())
                                }
                            },
                            Err(err) => {
                                let err = format!("Error getting downloads: {}", err);
                                tracing::error!("{}", err);
                                Err(err)
                            }
                        }
                    },
                    DownloadsMessage::UpdateDownloads,
                )
                .map(Message::DownloadsMessage)
            }
            Message::DownloadsMessage(message) => self
                .downloads
                .update(message)
                .map(Message::DownloadsMessage),
            Message::Resized(size) => self
                .config
                .ui
                .update(UpdateUiConfig::Size(size))
                .map(Message::UpdateUiConfig),
            Message::TitleBar(message) => self.title_bar.update(message).map(Message::TitleBar),
        }
    }

    pub fn settings_view(&self) -> Element<Message> {
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
                button(themed_icon(Icon::ArrowBack, FONT_SIZE_BODY * 1.25, None))
                    .style(button::text)
                    .on_press(Message::Navigate(Page::Downloading)),
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

    fn side_bar(&self, collapsed: bool) -> Element<Message> {
        let sidebar_button = |icon: Icon, button_text: String, page: Page| {
            let content = row![themed_icon(icon, FONT_SIZE_BODY * 1.25, None)];
            let button = button(
                if collapsed {
                    content
                } else {
                    content.push(text(button_text))
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
            .on_press(Message::Navigate(page))
            .style(button::text);
            if collapsed {
                button
            } else {
                button.width(FONT_SIZE_BODY * 12.)
            }
        };

        return container(
            column![
                sidebar_button(Icon::Downloading, "Downloading".into(), Page::Downloading),
                sidebar_button(Icon::Pause, "Waiting".into(), Page::Waiting),
                sidebar_button(Icon::Directory, "All Downloads".into(), Page::All),
                Space::new(0., Length::Fill),
                sidebar_button(Icon::Settings, "Settings".into(), Page::Settings),
            ]
            .padding(
                Padding::new(0.)
                    .top(FONT_SIZE_BODY * 1.25)
                    .bottom(FONT_SIZE_BODY * 1.25),
            ),
        )
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.scale_alpha(0.05).into()),
                ..Default::default()
            }
        })
        .into();
    }

    pub fn view(&self) -> Element<Message> {
        let mut view = column![];
        if self.config.ui.custom_decoration {
            view = view.push(
                self.title_bar
                    .view(self.config.ui.maximized)
                    .map(Message::TitleBar),
            );
        }
        let content = row![
            self.side_bar(self.config.ui.size.width < 1100.),
            match self.current_page {
                Page::All | Page::Downloading =>
                    self.downloads.view().map(Message::DownloadsMessage),
                Page::Settings => self.settings_view(),
                _ => text("404").into(),
            }
        ];
        view = view.push(content);
        view.into()
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
