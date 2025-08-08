use crate::{
    DOWNLOAD_MANAGER_TITLE,
    styles::constants::{BORDER_WIDTH, FONT_SIZE_BODY},
};
use iced::{
    Alignment, Border, Element, Length, Padding, Pixels, Task, Theme,
    widget::{button, column, container, mouse_area, row, shader::wgpu::naga::BOOL_WIDTH, text},
    window,
};

use super::icons::{Icon, themed_icon};

pub struct TitleBar {
    title: String,
    focused: bool,
}

#[derive(Debug, Clone)]
pub enum TitleBarMessage {
    DragStart,
    Focused,
    UnFocused,
    Minimize,
    ToggleMaximize(bool),
    Close,
    DoNothing,
}

impl Default for TitleBar {
    fn default() -> Self {
        Self {
            title: DOWNLOAD_MANAGER_TITLE.into(),
            focused: false,
        }
    }
}

impl TitleBar {
    pub fn update(&mut self, message: TitleBarMessage) -> Task<TitleBarMessage> {
        match message {
            TitleBarMessage::Close => window::get_latest().and_then(window::close),
            TitleBarMessage::DragStart => window::get_latest().and_then(window::drag),
            TitleBarMessage::Focused => {
                self.focused = true;
                Task::none()
            }
            TitleBarMessage::UnFocused => {
                self.focused = false;
                Task::none()
            }
            TitleBarMessage::Minimize => {
                window::get_latest().and_then(|id| window::minimize(id, true))
            }
            TitleBarMessage::ToggleMaximize(maximized) => {
                window::get_latest().and_then(move |id| window::maximize(id, !maximized))
            }
            TitleBarMessage::DoNothing => Task::none(),
        }
    }

    pub fn view(&self, maximized: bool) -> Element<TitleBarMessage> {
        let title_bar_button = |icon: Icon, on_press: TitleBarMessage| {
            button(themed_icon(icon, FONT_SIZE_BODY * 1.25, None))
                .padding(FONT_SIZE_BODY * 0.25)
                .style(|theme, status| {
                    let palette = theme.extended_palette();
                    let base = button::Style {
                        text_color: palette.background.base.text,
                        border: Border::rounded(Border::default(), FONT_SIZE_BODY),
                        ..Default::default()
                    };
                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(palette.background.weak.color.scale_alpha(0.1).into()),
                            ..base
                        },
                        _ => base,
                    }
                })
                .on_press(on_press)
        };

        let title_bar = container(
            row![
                text(&self.title),
                mouse_area(container("").width(Length::Fill)).on_press(TitleBarMessage::DragStart),
                row![
                    title_bar_button(Icon::Minimize, TitleBarMessage::Minimize),
                    title_bar_button(
                        if maximized {
                            Icon::Restore
                        } else {
                            Icon::Maximize
                        },
                        TitleBarMessage::ToggleMaximize(maximized)
                    ),
                    title_bar_button(Icon::Close, TitleBarMessage::Close),
                ]
                .spacing(FONT_SIZE_BODY * 0.5)
            ]
            .padding(Padding::new(FONT_SIZE_BODY).top(0.).bottom(0.))
            .height((FONT_SIZE_BODY * 3.) - BORDER_WIDTH)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(if !self.focused {
                    palette.background.weak.color.scale_alpha(0.025).into()
                } else {
                    palette.background.base.color.into()
                }),
                ..Default::default()
            }
        });

        let separator = container("")
            .width(Length::Fill)
            .height(Pixels::from(BOOL_WIDTH as u16))
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(palette.background.weak.color.scale_alpha(0.05).into()),
                    ..Default::default()
                }
            });

        column![title_bar, separator].into()
    }
}
