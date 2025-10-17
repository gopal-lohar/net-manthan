use crate::{
    components::icons::{Icon, themed_icon},
    styles::constants::{BORDER_ROUNDED_RADIUS, BORDER_WIDTH, FONT_SIZE_BODY},
};
use iced::{
    Alignment, Background, Border, Element, Length, Task, Theme,
    border::Radius,
    widget::{
        Column, Scrollable, Space, button, column, container, mouse_area, row,
        scrollable::{Direction, Scrollbar},
        text,
    },
};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct Toast {
    messages: Vec<ToastItem>,
}

#[derive(Debug, Clone)]
pub enum ToastVariant {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToastItem {
    id: u64,
    message: String,
    variant: ToastVariant,
}

impl Default for Toast {
    fn default() -> Self {
        Toast {
            messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ToastMessage {
    Show(String, ToastVariant),
    DeleteToast(u64),
    DoNothing,
}

impl Toast {
    pub fn update(&mut self, message: ToastMessage) -> Task<ToastMessage> {
        match message {
            ToastMessage::Show(msg, variant) => {
                let id = rand::random::<u64>();
                self.messages.push(ToastItem {
                    id,
                    variant,
                    message: msg.clone(),
                });
                Task::perform(
                    async move {
                        sleep(Duration::from_secs(5)).await;
                        ()
                    },
                    move |_| ToastMessage::DeleteToast(id),
                )
            }
            ToastMessage::DeleteToast(id) => {
                self.messages.retain(|toast| toast.id != id);
                Task::none()
            }
            ToastMessage::DoNothing => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, ToastMessage> {
        let mut toasts = Vec::new();

        for msg in &self.messages {
            toasts.push(
                mouse_area(
                    container(
                        container(
                            row![
                                row![
                                    themed_icon(Icon::Info, FONT_SIZE_BODY * 1.25, None),
                                    text(&msg.message).size(FONT_SIZE_BODY * 0.8)
                                ]
                                .padding(FONT_SIZE_BODY * 0.5)
                                .align_y(Alignment::Center)
                                .spacing(FONT_SIZE_BODY * 0.5)
                                .width(Length::Fill),
                                button(
                                    row![themed_icon(Icon::Close, FONT_SIZE_BODY * 1.5, None)]
                                        .align_y(Alignment::Center)
                                        .height(Length::from(FONT_SIZE_BODY * 2.))
                                )
                                .width(Length::from(FONT_SIZE_BODY * 2.))
                                .on_press(ToastMessage::DeleteToast(msg.id))
                                .style(button::text)
                            ]
                            .align_y(Alignment::Center),
                        )
                        .width(Length::Fill)
                        .style(move |theme: &Theme| {
                            iced::widget::container::Style {
                                background: Some(Background::from(match msg.variant {
                                    ToastVariant::Info => theme.palette().success.scale_alpha(0.01),
                                    ToastVariant::Error => theme.palette().danger.scale_alpha(0.01),
                                })),
                                border: Border {
                                    width: BORDER_WIDTH,
                                    color: theme.palette().text.scale_alpha(
                                        if theme.extended_palette().is_dark {
                                            0.01
                                        } else {
                                            0.1
                                        },
                                    ),
                                    radius: Radius::new(BORDER_ROUNDED_RADIUS),
                                },
                                ..Default::default()
                            }
                        }),
                    )
                    .width(FONT_SIZE_BODY * 30.)
                    .style(move |theme: &Theme| {
                        iced::widget::container::Style {
                            background: Some(Background::from(theme.palette().background)),
                            border: Border {
                                radius: Radius::new(BORDER_ROUNDED_RADIUS),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
                )
                .on_press(ToastMessage::DoNothing)
                .into(),
            );
        }

        column![
            Space::new(0., Length::FillPortion(1)),
            Scrollable::with_direction(
                Column::from_vec(toasts)
                    .spacing(FONT_SIZE_BODY * 0.25)
                    .padding(FONT_SIZE_BODY)
                    .width(Length::Fill)
                    .align_x(Alignment::End),
                Direction::Vertical(
                    Scrollbar::default().anchor(iced::widget::scrollable::Anchor::End)
                ),
            )
        ]
        .into()
    }
}
