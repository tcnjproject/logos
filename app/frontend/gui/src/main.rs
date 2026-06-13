// TCNJ AI/ML Group
mod app;
mod audio;
mod theme;
// mod tiles;
mod views;
mod data;

use app::Logos as Ui;
use iced::{application, Settings, window, Size};

fn title(_state: &Ui) -> String {
    format!("Logos v{}", env!("CARGO_PKG_VERSION"))
}

pub fn main() -> iced::Result {
    application(title, Ui::update, Ui::view)
        .subscription(Ui::subscription)
        .theme(Ui::theme)
        .window(window::Settings {
            size: Size::new(1400.0, 700.0),
            min_size: Some(Size::new(900.0, 600.0)),
            position: window::Position::Centered,
             icon: Some(window::icon::from_file("assets/logos.png").expect("Failed to load icon")),
            ..Default::default()
        })
        .settings(Settings {
            ..Default::default()
        })
        .run_with(Ui::new)
}