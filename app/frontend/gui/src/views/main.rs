// TCNJ AI/ML Group

use iced::{
    widget::{
        button, column, container, row, scrollable, svg, text, text_input,
        Space, Rule,
    },
    Alignment, Background, Border, Color, Element, Length,
};

use crate::app::{Message, Logos as Ui};
use crate::data::*;
use crate::theme::*;
use crate::views::components::*;

pub fn view(app: &Ui) -> Element<'_, Message> {
    let header = view_header(app);
    let body = view_body(app);

    let stack = column![header, body].spacing(0);

    // Overlay tour tooltip + update banner on top
    let _layers: Vec<Element<Message>> = vec![stack.into()];

    // Tour tooltip overlay
    if let Some(step) = &app.tour_step {
        let tooltip = tour_card(step);
        // Position it bottom-left-ish; iced doesn't have z-index so we use a
        // stack-like approach via the overlay module approach below.
        // We'll float it using a column push to bottom area.
        let _ = tooltip; // handled below in overlay
    }

    // Use iced's overlay-style layout: base content + floating overlay layer
    let base: Element<Message> = {
        let content = column![
            view_header(app), 
            view_body(app)]
            .spacing(0);
        content.into()
    };

    // Build full layout with overlays
    build_overlay(app, base)
}

fn build_overlay<'a>(app: &'a Ui, base: Element<'a, Message>) -> Element<'a, Message> {
    // We simulate overlays by using a Stack-like container.
    // iced 0.13 has iced::widget::stack for layering.
    use iced::widget::stack;

    let mut layers: Vec<Element<Message>> = vec![base];

    // Tour card overlay
    if let Some(step) = &app.tour_step {
        let card = tour_card(step);
        let positioned = container(
            column![
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(match step {
                        TourStep::LiveTranscript => 20u16,
                        TourStep::BookSearch | TourStep::ContextSearch => 20,
                        TourStep::Broadcast => 1050,
                        _ => 700,
                    }),
                    card,
                ]
                .spacing(0),
                Space::with_height(match step {
                    TourStep::Broadcast => 80u16,
                    _ => 120,
                }),
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        layers.push(positioned.into());
    }

    // Update banner
    if app.show_update_banner && app.tour_step.is_none() {
        let banner = update_banner();
        let positioned = container(
            column![
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    banner,
                    Space::with_width(20),
                ],
                Space::with_height(20),
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        layers.push(positioned.into());
    }

    stack(layers).into()
}

fn view_header(_app: &Ui) -> Element<'_, Message> {
    let logo = row![
        // text("⚡").size(20).color(ACCENT),
        // image(image::Handle::from_path("assets/tcnj-logo.png"))
        //     .width(60)
        //     .height(60),
        text("LOGOS").size(16).color(ACCENT),
        Space::with_width(8),
        container(text("Beta").size(10).color(TEXT_MUTED))
            .padding([2, 6])
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(BG_PANEL)),
                border: Border {
                    color: BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(6);

    // let timer_pill = container(
    //     text(format!("REMAINING: {}", app.remaining_formatted()))
    //         .size(13)
    //         .color(TEXT_SECONDARY),
    // )
    // .padding([5, 14])
    // .style(|_: &iced::Theme| iced::widget::container::Style {
    //     background: Some(Background::Color(BG_PANEL)),
    //     border: Border {
    //         color: BORDER,
    //         width: 1.0,
    //         radius: 14.0.into(),
    //     },
    //     ..Default::default()
    // });

    let toolbar = row![
        // toolbar_button("📋", Message::OpenSettings),
        // toolbar_button("📺", Message::OpenBroadcast),
        // toolbar_button("🌐", Message::OpenDisplay),
        toolbar_button_with_image("assets/help.svg", Message::OpenAboutLogos),
        toolbar_button_with_image("assets/settings.svg", Message::OpenSettings),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    container(
        row![
            Space::with_width(Length::Fill),
            // timer_pill,
            logo,
            Space::with_width(Length::Fill),
            toolbar,
        ]
        .align_y(Alignment::Center)
        .padding([8, 16]),
    )
    .width(Length::Fill)
    .style(|_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_DARK)),
        border: Border {
            color: BORDER,
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn view_body(app: &Ui) -> Element<'_, Message> {
    row![
        // Col 1: Live Transcript
        view_transcript_panel(app),
        // Col 2: Bible Search + Preview
        column![
            row![
                view_preview_panel(app),
                view_live_panel(app),
            ]
            .spacing(10),
             view_search_area(app)
        ]
        .spacing(10)
        // .padding([10,10])
        .height(Length::Fill),
        
       
        // Col 4: Queue + Recent Detections
        view_right_column(app),
    ]
    .spacing(10)
    .padding([10, 10])
    .height(Length::Fill)
    .into()
}

// ─── PANEL 1: Live Transcript ────────────────────────────────────────────────

fn view_transcript_panel(app: &Ui) -> Element<'_, Message> {
    // use crate::views::waveform::Waveform;
    use crate::views::waveform::VuMeter;

    let is_active = app.tour_step == Some(TourStep::LiveTranscript);

    // ── Waveform / level display ──────────────────────────────────────────────
    // While recording: live bar-graph driven by real mic data.
    // While idle: flat bars at zero amplitude.
    // let waveform_widget = Waveform::new(
    //     &app.waveform,
    //     app.is_transcribing,
    //     app.audio_peak,
    // )
    
    // ── VU meter display ──────────────────────────────────────────────────────
    // Live segmented L/R meter when recording; dim idle state otherwise.
    let vu_widget = VuMeter::from_waveform(
        &app.waveform,
        app.audio_peak,
        app.is_transcribing,
    )
    .view();
 
    // RMS level label — only show when recording
    let _level_label: Element<Message> = if app.is_transcribing {
        let db = if app.audio_rms > 1e-6 {
            20.0 * app.audio_rms.log10()
        } else {
            -60.0
        };
        text(format!("{:.0} dB", db.max(-60.0)))
            .size(11)
            .color(TEXT_MUTED)
            .into()
    } else {
        Space::with_height(0).into()
    };

     // ── Transcript text area ─────
    let mic_icon = svg(svg::Handle::from_path(if app.is_transcribing {
        "assets/mic_on.svg"
    } else {
        "assets/mic_off.svg"
    }))
    .width(24)
    .height(24);

    let transcript_area: Element<Message> = if app.transcript_text.is_empty() {
        column![
            Space::with_height(Length::Fill),
            container(mic_icon).center(Length::Fill),
            Space::with_height(10),
            text("Click below to start live transcription")
                .size(13)
                .color(TEXT_SECONDARY),
            text("and follow along in real time")
                .size(13)
                .color(TEXT_SECONDARY),
            Space::with_height(Length::Fill),
        ]
        .align_x(Alignment::Center)
        .spacing(4)
        .into()
    } else {
        scrollable(
            text(&app.transcript_text).size(13).color(TEXT_PRIMARY),
        )
        .height(Length::Fill)
        .into()
    };

    // Audio level indicator (3 bars)
    let level_bars = row![
        level_bar(0.6, app.is_transcribing),
        level_bar(0.8, app.is_transcribing),
        level_bar(0.4, app.is_transcribing),
        level_bar(1.0, app.is_transcribing),
        level_bar(0.5, app.is_transcribing),
    ]
    .spacing(2);

    let start_btn = button(
        row![
            svg(svg::Handle::from_path(if app.is_transcribing {
                "assets/stop.svg"
            } else {
                "assets/start.svg"
            }))
            .width(24)
            .height(24)
            .style(|_: &iced::Theme, _| svg::Style {
                color: Some(ACCENT),
                ..Default::default()
            }),
            Space::with_width(6),
            text(if app.is_transcribing {
                "Stop transcribing"
            } else {
                "Start transcribing"
            })
            .size(13)
            .color(ACCENT),
        ]
        .align_y(Alignment::Center),
    )
    .padding([8, 0])
    .style(|_: &iced::Theme, _| button::Style {
        background: None,
        ..Default::default()
    })
    .on_press(Message::ToggleTranscription);

    let header = row![
        text("Live transcript").size(13).color(TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        level_bars,
    ]
    .align_y(Alignment::Center);

    let meter_side = container(vu_widget)
        .width(Length::Fixed(20.0))
        .height(Length::Fill)
        .align_x(Alignment::End)
        .padding([0, 0])
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: None,
            border: Border { width: 0.0, ..Default::default() },
            ..Default::default()
        });

    let inner = column![
        header,
        Space::with_height(8),
        row![
            container(transcript_area)
                .width(Length::Fill)
                .height(Length::Fill),
            meter_side,
        ]
        .spacing(8)
        .height(Length::Fill),
        Rule::horizontal(1),
        Space::with_height(4),
        start_btn,
    ]
    .spacing(0)
    .height(Length::Fill);

    let border_color = if is_active { ACCENT } else { BORDER };

    container(inner)
        .width(280)
        .height(Length::Fill)
        .padding(12)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BG_PANEL)),
            border: Border {
                color: border_color,
                width: if is_active { 2.0 } else { 1.0 },
                radius: 5.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn level_bar(height_frac: f32, active: bool) -> Element<'static, Message> {
    let h = (height_frac * 16.0).max(3.0) as u16;
    let color = if active { ACCENT } else { TEXT_MUTED };

    container(Space::with_height(0))
        .width(3)
        .height(h)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// ─── PANEL 2: Bible Search (center column) ───────────────────────────────────

fn view_preview_panel(app: &Ui) -> Element<'_, Message> {
    // Top: Program preview
    let preview_label = text("Program preview").size(13).color(TEXT_SECONDARY);
    let preview_is_active = app.tour_step == Some(TourStep::Preview);

    let preview_box = container(preview_screen(app.preview_verse.as_ref()))
        .width(Length::Fill)
        .height(220)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BG_PANEL)),
            border: Border {
                color: if preview_is_active { ACCENT } else { BORDER },
                width: if preview_is_active { 2.0 } else { 1.0 },
                radius: 8.0.into(),
            },
            ..Default::default()
        });

    // Bottom: search bar + results
    // let search_area = view_search_area(app);

    let content = column![
        container(
            column![
                preview_label,
                Space::with_height(8),
                preview_box,
            ]
        )
        .padding(iced::Padding { top: 12.0, right: 12.0, bottom: 0.0, left: 12.0 }),
        // search_area,
    ]
    .spacing(0)
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BG_DARK)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_search_area(app: &Ui) -> Element<'_, Message> {
    let book_active = app.search_mode == SearchMode::Book;

    let tabs = row![
        tab_button(
            "Book search",
            "📖",
            book_active,
            Message::SearchModeChanged(SearchMode::Book)
        ),
        tab_button(
            "Context search",
            "\"\"",
            !book_active,
            Message::SearchModeChanged(SearchMode::Context)
        ),
    ]
    .spacing(4);

    let placeholder = if book_active {
        "Jonah 2:8 or keyword"
    } else {
        "Search by theme, quote, or paraphrase..."
    };

    let search_input = text_input(placeholder, &app.search_query)
        .size(13)
        .padding([8, 12])
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::SearchSubmitted)
        .style(|_: &iced::Theme, _| text_input::Style {
            background: Background::Color(BG_INPUT),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: TEXT_MUTED,
            placeholder: TEXT_MUTED,
            value: TEXT_PRIMARY,
            selection: ACCENT,
        });

    // Translation dropdown
    let translation_btn = button(
        row![
            text(app.translation.label()).size(13).color(TEXT_PRIMARY),
            Space::with_width(4),
            text("▾").size(11).color(TEXT_SECONDARY),
        ]
        .align_y(Alignment::Center),
    )
    .padding([8, 10])
    .style(|_: &iced::Theme, status| {
        let bg = match status {
            button::Status::Hovered => BG_PANEL_LIGHT,
            _ => BG_INPUT,
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
    .on_press(Message::TranslationDropdownToggled);

    let search_row = row![
        search_input,
        Space::with_width(8),
        translation_btn,
    ]
    .align_y(Alignment::Center);

    // Translation dropdown menu
    let dropdown: Element<Message> = if app.translation_dropdown_open {
        container(
            column(
                Translation::all()
                    .into_iter()
                    .map(|t| {
                        let label = t.label().to_string();
                        let is_sel = t == app.translation;
                        button(
                            text(label).size(13).color(if is_sel { ACCENT } else { TEXT_PRIMARY }),
                        )
                        .padding([8, 14])
                        .width(Length::Fill)
                        .style(move |_: &iced::Theme, status| {
                            let bg = match status {
                                button::Status::Hovered => BG_PANEL_LIGHT,
                                _ => if is_sel { BG_PANEL } else { BG_INPUT },
                            };
                            button::Style {
                                background: Some(Background::Color(bg)),
                                text_color: if is_sel { ACCENT } else { TEXT_PRIMARY },
                                ..Default::default()
                            }
                        })
                        .on_press(Message::TranslationChanged(t))
                        .into()
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(0),
        )
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BG_INPUT)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .width(120)
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Search results or empty state
    let results_area: Element<Message> = if app.search_results.is_empty() {
        container(
            column![
                Space::with_height(Length::Fill),
                container(text("📖").size(32).color(TEXT_MUTED)).center(Length::Fill),
                Space::with_height(8),
                text("No scripture selected").size(14).color(TEXT_SECONDARY),
                text("Search for a book, chapter, or keyword above")
                    .size(12)
                    .color(TEXT_MUTED),
                Space::with_height(Length::Fill),
            ]
            .align_x(Alignment::Center)
            .spacing(4),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        scrollable(
            column(
                app.search_results
                    .iter()
                    .map(|v| verse_result_row(v))
                    .collect::<Vec<_>>(),
            )
            .spacing(1),
        )
        .height(Length::Fill)
        .into()
    };

    let content =column![
        container(
            column![
                tabs,
                Space::with_height(8),
                search_row,
                {
                    let dropdown_row: Element<Message> = if app.translation_dropdown_open {
                        row![
                            Space::with_width(Length::Fill),
                            dropdown,
                            Space::with_width(0),
                        ]
                        .into()
                    } else {
                        Space::with_height(0).into()
                    };
                    dropdown_row
                },
            ]
            .spacing(4),
        )
        .padding(iced::Padding { top: 12.0, right: 12.0, bottom: 8.0, left: 12.0 }),
        Rule::horizontal(1),
        results_area,
    ]
    .spacing(0)
    .height(Length::Fill);

    container(content)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BG_DARK)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        })
        .into()
   
}

fn verse_result_row(verse: &Verse) -> Element<'_, Message> {
    let v = verse.clone();
    let v2 = verse.clone();

    container(
        row![
            column![
                text(&verse.reference).size(13).color(ACCENT),
                Space::with_height(3),
                text(&verse.text).size(12).color(TEXT_SECONDARY),
            ]
            .spacing(0)
            .width(Length::Fill),
            Space::with_width(8),
            column![
                button(text("Present").size(11).color(Color::WHITE))
                    .padding([4, 8])
                    .style(|_: &iced::Theme, status| {
                        let bg = match status {
                            button::Status::Hovered => ACCENT_HOVER,
                            _ => ACCENT,
                        };
                        button::Style {
                            background: Some(Background::Color(bg)),
                            border: Border { radius: 4.0.into(), ..Default::default() },
                            text_color: Color::WHITE,
                            ..Default::default()
                        }
                    })
                    .on_press(Message::PresentVerse(v)),
                Space::with_height(4),
                button(text("Queue").size(11).color(TEXT_PRIMARY))
                    .padding([4, 8])
                    .style(|_: &iced::Theme, _| button::Style {
                        background: Some(Background::Color(BG_PANEL_LIGHT)),
                        border: Border {
                            color: BORDER,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        text_color: TEXT_PRIMARY,
                        ..Default::default()
                    })
                    .on_press(Message::AddToQueue(v2)),
            ]
            .spacing(0),
        ]
        .align_y(Alignment::Start),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(|_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL)),
        border: Border {
            color: BORDER,
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

// ─── PANEL 3: Live Display ───────────────────────────────────────────────────

fn view_live_panel(app: &Ui) -> Element<'_, Message> {
    let is_active = app.tour_step == Some(TourStep::LiveOutput);

    let header = row![
        text("Live display").size(13).color(TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text("Go live").size(13).color(TEXT_SECONDARY),
        Space::with_width(8),
        toggle_switch(app.go_live, Message::GoLiveToggled),
    ]
    .align_y(Alignment::Center);

    let live_screen = container(preview_screen(app.live_verse.as_ref()))
        .width(Length::Fill)
        .height(220)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BLACK)),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let inner = column![
        header,
        Space::with_height(8),
        live_screen,
        Space::with_height(8),
        if app.live_verse.is_some() {
            let r: Element<Message> = row![
                Space::with_width(Length::Fill),
                ghost_button("Clear live", Message::ClearLive),
            ]
            .into();
            r
        } else {
            Space::with_height(0).into()
        },
    ]
    .spacing(0)
    .height(Length::Fill);

    let border_color = if is_active { ACCENT } else { BORDER };

    container(inner)
        .width(380)
        .height(Length::Fill)
        .padding(12)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BG_PANEL)),
            border: Border {
                color: border_color,
                width: if is_active { 2.0 } else { 1.0 },
                radius: 5.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ─── PANEL 4: Queue + Recent Detections ──────────────────────────────────────

fn view_right_column(app: &Ui) -> Element<'_, Message> {
    let queue_is_active = app.tour_step == Some(TourStep::VerseQueue);
    let detect_is_active = app.tour_step == Some(TourStep::AiDetections);

    // Queue section
    let queue_content: Element<Message> = if app.queue.is_empty() {
        column![
            Space::with_height(20),
            text("No verses in queue").size(13).color(TEXT_SECONDARY),
            Space::with_height(4),
            text("Add verses from the Bible reader").size(12).color(TEXT_MUTED),
        ]
        .align_x(Alignment::Center)
        .spacing(0)
        .into()
    } else {
        scrollable(
            column(
                app.queue
                    .iter()
                    .enumerate()
                    .map(|(i, v)| queue_item(v, i))
                    .collect::<Vec<_>>(),
            )
            .spacing(4),
        )
        .height(150)
        .into()
    };

    let clear_btn: Element<Message> = if !app.queue.is_empty() {
        button(text("Clear").size(11).color(TEXT_MUTED))
            .padding([2, 6])
            .style(|_: &iced::Theme, _| button::Style {
                background: None,
                ..Default::default()
            })
            .on_press(Message::ClearQueue)
            .into()
    } else {
        Space::with_width(0).into()
    };

    let queue_header = row![
        text("Queue").size(13).color(TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        clear_btn,
    ]
    .align_y(Alignment::Center);

    let queue_panel = container(
        column![
            queue_header,
            Space::with_height(8),
            Rule::horizontal(1),
            Space::with_height(8),
            queue_content,
        ]
        .spacing(0),
    )
    .padding(12)
    .style(move |_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL)),
        border: Border {
            color: if queue_is_active { ACCENT } else { BORDER },
            width: if queue_is_active { 2.0 } else { 1.0 },
            radius: 5.0.into(),
        },
        ..Default::default()
    });

    // Recent detections section
    let detections_content: Element<Message> = if app.recent_detections.is_empty() {
        column![
            Space::with_height(20),
            text("No detections yet").size(13).color(TEXT_SECONDARY),
            Space::with_height(4),
            text("Verses detected from speech appear here")
                .size(12)
                .color(TEXT_MUTED),
        ]
        .align_x(Alignment::Center)
        .spacing(0)
        .into()
    } else {
        scrollable(
            column(
                app.recent_detections
                    .iter()
                    .map(|v| detection_item(v))
                    .collect::<Vec<_>>(),
            )
            .spacing(4),
        )
        .into()
    };

    let detect_header = text("Recent detections").size(13).color(TEXT_SECONDARY);

    let detect_panel = container(
        column![
            detect_header,
            Space::with_height(8),
            Rule::horizontal(1),
            Space::with_height(8),
            detections_content,
        ]
        .spacing(0)
        .height(Length::Fill),
    )
    .padding(12)
    .height(Length::Fill)
    .style(move |_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL)),
        border: Border {
            color: if detect_is_active { ACCENT } else { BORDER },
            width: if detect_is_active { 2.0 } else { 1.0 },
            radius: 5.0.into(),
        },
        ..Default::default()
    });

    column![
        queue_panel,
        detect_panel,
    ]
    .spacing(10)
    .width(280)
    .height(Length::Fill)
    .into()
}

fn queue_item(verse: &Verse, idx: usize) -> Element<'_, Message> {
    let v = verse.clone();

    container(
        row![
            column![
                text(&verse.reference).size(12).color(ACCENT),
                text(&verse.text).size(11).color(TEXT_SECONDARY),
            ]
            .spacing(2)
            .width(Length::Fill),
            Space::with_width(8),
            column![
                button(text("▶").size(11).color(Color::WHITE))
                    .padding([3, 6])
                    .style(|_: &iced::Theme, _| button::Style {
                        background: Some(Background::Color(ACCENT)),
                        border: Border { radius: 4.0.into(), ..Default::default() },
                        text_color: Color::WHITE,
                        ..Default::default()
                    })
                    .on_press(Message::PresentVerse(v)),
                Space::with_height(3),
                button(text("✕").size(10).color(TEXT_MUTED))
                    .padding([3, 6])
                    .style(|_: &iced::Theme, _| button::Style {
                        background: Some(Background::Color(BG_PANEL_LIGHT)),
                        border: Border {
                            color: BORDER,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .on_press(Message::RemoveFromQueue(idx)),
            ],
        ]
        .align_y(Alignment::Start),
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(|_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL_LIGHT)),
        border: Border { radius: 6.0.into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}

fn detection_item(verse: &Verse) -> Element<'_, Message> {
    let v = verse.clone();
    let v2 = verse.clone();

    container(
        row![
            column![
                text(&verse.reference).size(12).color(ACCENT),
                text(&verse.text).size(11).color(TEXT_SECONDARY),
            ]
            .spacing(2)
            .width(Length::Fill),
            Space::with_width(8),
            column![
                button(text("Present").size(10).color(Color::WHITE))
                    .padding([3, 7])
                    .style(|_: &iced::Theme, _| button::Style {
                        background: Some(Background::Color(ACCENT)),
                        border: Border { radius: 4.0.into(), ..Default::default() },
                        text_color: Color::WHITE,
                        ..Default::default()
                    })
                    .on_press(Message::PresentVerse(v)),
                Space::with_height(3),
                button(text("Queue").size(10).color(TEXT_PRIMARY))
                    .padding([3, 7])
                    .style(|_: &iced::Theme, _| button::Style {
                        background: Some(Background::Color(BG_PANEL_LIGHT)),
                        border: Border {
                            color: BORDER,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        text_color: TEXT_PRIMARY,
                        ..Default::default()
                    })
                    .on_press(Message::AddToQueue(v2)),
            ],
        ]
        .align_y(Alignment::Start),
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(|_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL_LIGHT)),
        border: Border { radius: 6.0.into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}