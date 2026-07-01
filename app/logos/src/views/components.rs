// TCNJ AI/ML Group

use iced::{
    widget::{button, container, text, row, column, Space, svg},
    Element, Length, Alignment, Border, Color, Background, Theme,
};

use crate::app::{self, Message};
use crate::theme::*;

/// Panel container with dark background and border
// pub fn panel<'a>(
//     content: impl Into<Element<'a, Message>>,
// ) -> Element<'a, Message> {
//     container(content)
//         .padding(12)
//         .style(|_: &Theme| iced::widget::container::Style {
//             background: Some(Background::Color(BG_PANEL)),
//             border: Border {
//                 color: BORDER,
//                 width: 1.0,
//                 radius: 8.0.into(),
//             },
//             ..Default::default()
//         })
//         .into()
// }

// /// Section header label
// pub fn section_label(label: &str) -> Element<'static, Message> {
//     text(label.to_string())
//         .size(13)
//         .color(TEXT_SECONDARY)
//         .into()
// }

/// Orange accent button — takes &str and converts to owned String to satisfy 'static
pub fn accent_button(label: &str, msg: Message) -> Element<'static, Message> {
    let label = label.to_string();
    button(
        text(label).size(13).color(Color::WHITE),
    )
    .padding([6, 14])
    .style(|_: &Theme, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => ACCENT_HOVER,
            _ => ACCENT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            border: Border { radius: 6.0.into(), ..Default::default() },
            text_color: Color::WHITE,
            ..Default::default()
        }
    })
    .on_press(msg)
    .into()
}

/// Ghost / secondary button
pub fn ghost_button(label: &str, msg: Message) -> Element<'static, Message> {
    let label = label.to_string();
    button(
        text(label).size(13).color(TEXT_PRIMARY),
    )
    .padding([6, 14])
    .style(|_: &Theme, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => BG_PANEL_LIGHT,
            _ => BG_PANEL,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: TEXT_PRIMARY,
            ..Default::default()
        }
    })
    .on_press(msg)
    .into()
}

/// Icon-style toolbar button
pub fn toolbar_button_with_image(icon: &str, msg: Message) -> Element<'static, Message> {
    button(
            svg(svg::Handle::from_path(icon))
                .width(24)
                .height(24),
        )
        .padding([6, 10])
        .style(|_: &iced::Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => BG_PANEL,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 6.0.into(), ..Default::default() },
                text_color: TEXT_SECONDARY,
                ..Default::default()
            }
        })
        .on_press(msg)
        .into()
}
/// Black preview screen area
pub fn preview_screen<'a>(
    verse: Option<&'a crate::data::Verse>,
) -> Element<'a, Message> {
    let inner: Element<'a, Message> = if let Some(v) = verse {
        column![
            text(&v.reference).size(13).color(TEXT_SECONDARY),
            Space::with_height(8),
            text(&v.text).size(16).color(TEXT_PRIMARY),
        ]
        .spacing(4)
        .padding(20)
        .into()
    } else {
        Space::with_height(0).into()
    };

    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(BLACK)),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Toggle switch widget
pub fn toggle_switch(is_on: bool, msg: Message) -> Element<'static, Message> {
    let color = if is_on { GREEN_LIVE } else { BG_PANEL_LIGHT };
    let label = if is_on { "●" } else { "○" };

    button(
        row![
            text(label).size(12).color(Color::WHITE),
            Space::with_width(4),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 10])
    .style(move |_: &Theme, _| button::Style {
        background: Some(Background::Color(color)),
        border: Border { radius: 12.0.into(), ..Default::default() },
        text_color: Color::WHITE,
        ..Default::default()
    })
    .on_press(msg)
    .into()
}

/// Tab button (Book search / Context search)
pub fn tab_button(
    label: &str,
    icon: &str,
    is_active: bool,
    msg: Message,
) -> Element<'static, Message> {
    let bg = if is_active { ACCENT } else { BG_PANEL };
    let text_color = if is_active { Color::WHITE } else { TEXT_SECONDARY };
    let label = label.to_string();
    let icon = icon.to_string();

    button(
        row![
            text(icon).size(13).color(text_color),
            Space::with_width(6),
            text(label).size(13).color(text_color),
        ]
        .align_y(Alignment::Center),
    )
    .padding([8, 14])
    .style(move |_: &Theme, status| {
        let actual_bg = match status {
            button::Status::Hovered if !is_active => BG_PANEL_LIGHT,
            _ => bg,
        };
        button::Style {
            background: Some(Background::Color(actual_bg)),
            border: Border {
                color: if is_active { ACCENT } else { BORDER },
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color,
            ..Default::default()
        }
    })
    .on_press(msg)
    .into()
}

/// Tour tooltip card
pub fn tour_card<'a>(
    step: &'a crate::data::TourStep,
) -> Element<'a, Message> {
    let title_row = row![
        text("✦").size(16).color(Color::from_rgb(1.0, 0.8, 0.0)),
        Space::with_width(8),
        text(step.title()).size(15).color(TEXT_PRIMARY),
    ]
    .align_y(Alignment::Center);

    let desc = text(step.description())
        .size(13)
        .color(TEXT_SECONDARY);

    let progress = format!("{}/{}", step.index(), crate::data::TourStep::total());

    let back_btn: Element<Message> = if step.prev().is_some() {
        ghost_button("←", Message::TourBack)
    } else {
        Space::with_width(0).into()
    };

    let buttons = row![
        back_btn,
        Space::with_width(8),
        accent_button("Next", Message::TourNext),
        Space::with_width(Length::Fill),
        button(text(format!("Skip  {}", progress)).size(12).color(TEXT_MUTED))
            .padding([4, 0])
            .style(|_: &Theme, _| button::Style {
                background: None,
                text_color: TEXT_MUTED,
                ..Default::default()
            })
            .on_press(Message::TourSkip),
    ]
    .align_y(Alignment::Center);

    container(
        column![
            title_row,
            Space::with_height(10),
            desc,
            Space::with_height(18),
            buttons,
        ]
        .spacing(0),
    )
    .padding(18)
    .width(320)
    .style(|_: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(TOOLTIP_BG)),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    })
    .into()
}

/// Update banner
pub fn update_banner() -> Element<'static, Message> {
    container(
        row![
            column![
                text("Update Available").size(14).color(TEXT_PRIMARY),
                text(format!("Version {} is ready to install.", app::LATEST_VERSION)).size(12).color(TEXT_SECONDARY),
            ],
            Space::with_width(Length::Fill),
            accent_button("Install & Restart", Message::InstallUpdate),
        ]
        .align_y(Alignment::Center)
        .spacing(12),
    )
    .padding([12u16, 16])
    .width(360)
    .style(|_: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL_LIGHT)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    })
    .into()
}