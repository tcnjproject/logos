// TCNJ AI/ML Group

use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;
use crate::theme::{ACCENT, BG_PANEL_LIGHT, TEXT_MUTED};

/// Bar-graph waveform visualiser.
/// `bars` — slice of normalised amplitudes in [0.0, 1.0], one per bar.
/// `active` — when false, all bars render in muted grey.
pub struct Waveform<'a> {
    bars: &'a [f32],
    active: bool,
    peak: f32,
}

impl<'a> Waveform<'a> {
    pub fn new(bars: &'a [f32], active: bool, peak: f32) -> Self {
        Self { bars, active, peak }
    }

    pub fn view(self) -> Element<'a, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(60)
            .into()
    }
}

impl canvas::Program<Message> for Waveform<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let n = self.bars.len().max(1);
        let total_w = bounds.width;
        let total_h = bounds.height;

        // Bar geometry — leave 1px gaps between bars
        let gap = 1.5_f32;
        let bar_w = ((total_w - gap * (n as f32 - 1.0)) / n as f32).max(1.0);

        // Minimum bar height so silent bars are still visible
        let min_h = 3.0_f32;

        let bar_color = if self.active { ACCENT } else { TEXT_MUTED };
        let bg_color = BG_PANEL_LIGHT;

        for (i, &amp) in self.bars.iter().enumerate() {
            let x = i as f32 * (bar_w + gap);

            // Background slot (full height, dim)
            let bg_path = Path::rectangle(
                Point::new(x, 0.0),
                Size::new(bar_w, total_h),
            );
            frame.fill(&bg_path, bg_color);

            // Active bar — grows from the bottom
            let bar_h = (amp * total_h).max(min_h);
            let bar_y = total_h - bar_h;

            let bar_path = Path::rectangle(
                Point::new(x, bar_y),
                Size::new(bar_w, bar_h),
            );

            // Brighten the colour slightly for taller bars
            let brightness = 0.7 + amp * 0.3;
            let c = Color {
                r: (bar_color.r * brightness).min(1.0),
                g: (bar_color.g * brightness).min(1.0),
                b: (bar_color.b * brightness).min(1.0),
                a: bar_color.a,
            };
            frame.fill(&bar_path, c);
        }

        // Peak indicator — a thin horizontal line at the peak level
        if self.active && self.peak > 0.01 {
            let peak_y = total_h - (self.peak * total_h).min(total_h - 1.0);
            let peak_line = Path::line(
                Point::new(0.0, peak_y),
                Point::new(total_w, peak_y),
            );
            frame.stroke(
                &peak_line,
                canvas::Stroke {
                    style: canvas::stroke::Style::Solid(Color {
                        r: 1.0,
                        g: 0.6,
                        b: 0.2,
                        a: 0.6,
                    }),
                    width: 1.0,
                    ..Default::default()
                },
            );
        }

        vec![frame.into_geometry()]
    }
}