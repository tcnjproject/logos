// TCNJ AI/ML Group

use iced::{
    widget::{column, container, image, progress_bar, row, text, Space},
    Alignment, Element, Length,
};

use crate::app::Message;
use crate::theme::*;

pub fn view(progress: f32) -> Element<'static, Message> {
    let image = container(
        image("assets/tcnj-logo.png")
            .width(406)
            .height(200),
    );

    let logo = row![
        text("⚡").size(36).color(ACCENT),
        text("LOGOS").size(36).color(ACCENT),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let bar = container(
        progress_bar(0.0..=1.0, progress)
            .height(4)
            .style(|_theme: &iced::Theme| {
                iced::widget::progress_bar::Style {
                    background: iced::Background::Color(BG_PANEL_LIGHT),
                    bar: iced::Background::Color(ACCENT),
                    border: iced::Border::default(),
                }
            }),
    )
    .width(300);

    let label = text("Setting things up")
        .size(14)
        .color(TEXT_SECONDARY);

    let content = column![
        image,
        Space::with_height(16),
        logo,
        Space::with_height(24),
        bar,
        Space::with_height(10),
        label,
    ]
    .spacing(0)
    .align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(BG_DARK)),
            ..Default::default()
        })
        .into()
}