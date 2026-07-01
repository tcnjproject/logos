// TCNJ AI/ML Group

//! vMix-style segmented VU meter.
//!
//! Two vertical channel strips (L / R) made of coloured segments:
//!   green  — 0 % … 70 %   (signal present)
//!   yellow — 70 % … 90 %  (loud)
//!   red    — 90 % … 100 % (clip zone)
//!
//! A white peak-hold tick floats at the highest recent level on each channel
//! and decays slowly over time.

use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

// ── Palette ───────────────────────────────────────────────────────────────────
const SEG_OFF:    Color = Color { r: 0.10, g: 0.10, b: 0.10, a: 1.0 };
const SEG_GREEN:  Color = Color { r: 0.13, g: 0.80, b: 0.25, a: 1.0 };
const SEG_YELLOW: Color = Color { r: 0.95, g: 0.82, b: 0.10, a: 1.0 };
const SEG_RED:    Color = Color { r: 0.95, g: 0.18, b: 0.12, a: 1.0 };
const PEAK_COLOR: Color = Color { r: 1.00, g: 1.00, b: 1.00, a: 0.90 };

/// Total number of segments per channel strip.
const SEGMENTS: usize = 14;
/// Segment boundary fractions (0.0 – 1.0 of total height)
const GREEN_END:  f32 = 0.70;
const YELLOW_END: f32 = 0.90;
// above YELLOW_END → red

/// Gap between adjacent segments, in pixels.
const SEG_GAP: f32 = 1.0;
/// Gap between the two channel strips.
const CHAN_GAP: f32 = 1.6;
/// Horizontal padding inside the widget.
const H_PAD: f32 = 1.0;

pub struct VuMeter<'a> {
    /// Left-channel amplitude 0.0–1.0
    left:  f32,
    /// Right-channel amplitude 0.0–1.0 (mirrors left when mono)
    right: f32,
    /// Peak-hold level for L channel
    peak_l: f32,
    /// Peak-hold level for R channel
    peak_r: f32,
    /// Whether the meter is active (shows live levels vs dim/idle)
    active: bool,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> VuMeter<'a> {
    /// Create from a flat waveform buffer.
    /// Splits the buffer in half: first half → L, second half → R.
    pub fn from_waveform(bars: &[f32], peak: f32, active: bool) -> Self {
        let (left, right) = if bars.is_empty() {
            (0.0, 0.0)
        } else {
            let mid = bars.len() / 2;
            let l = rms(&bars[..mid.max(1)]);
            let r = rms(&bars[mid..]);
            (l, r)
        };
        // Soft-scale: the RMS of the downsampled bars is already normalised,
        // but VU meters look best with a little extra gain.
        let gain = 2.5_f32;
        Self {
            left:   (left  * gain).min(1.0),
            right:  (right * gain).min(1.0),
            peak_l: (peak  * gain).min(1.0),
            peak_r: (peak  * gain).min(1.0),
            active,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        Canvas::new(self)
            .width(Length::Fixed(10.0))
            .height(Length::Fill)
            .into()
    }
}

fn rms(s: &[f32]) -> f32 {
    if s.is_empty() { return 0.0; }
    (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
}

impl canvas::Program<Message> for VuMeter<'_> {
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

        let w = bounds.width;
        let h = bounds.height;

        let meter_h = h;

        // Each channel strip width
        let strip_w = (w - H_PAD * 2.0 - CHAN_GAP) / 2.0;
        let x_l = H_PAD;
        let x_r = H_PAD + strip_w + CHAN_GAP;

        draw_strip(&mut frame, x_l, 0.0, strip_w, meter_h,
                   self.left,  self.peak_l, self.active);
        draw_strip(&mut frame, x_r, 0.0, strip_w, meter_h,
                   self.right, self.peak_r, self.active);


        vec![frame.into_geometry()]
    }
}

/// Draw one vertical channel strip of SEGMENTS segments.
fn draw_strip(
    frame: &mut Frame,
    x: f32, y: f32,
    w: f32, h: f32,
    level: f32,
    peak: f32,
    active: bool,
) {
    let seg_h = (h - SEG_GAP * (SEGMENTS as f32 - 1.0)) / SEGMENTS as f32;

    for i in 0..SEGMENTS {
        // Segments run bottom-to-top; i=0 is the bottom segment.
        let frac = i as f32 / SEGMENTS as f32; // 0 (bottom) … 1 (top)

        let seg_y = y + h - (i as f32 + 1.0) * (seg_h + SEG_GAP) + SEG_GAP;

        let lit = active && level >= frac;

        let color = if !active {
            SEG_OFF
        } else if lit {
            seg_color(frac)
        } else {
            dim(seg_color(frac))
        };

        let rect = Path::rectangle(
            Point::new(x, seg_y),
            Size::new(w, seg_h),
        );
        frame.fill(&rect, color);
    }

    // Peak-hold tick — a bright 2px bar at the peak segment
    if active && peak > 0.01 {
        let peak_frac = peak.min(0.999);
        // Find which segment row this falls in
        let seg_idx = (peak_frac * SEGMENTS as f32) as usize;
        let seg_idx = seg_idx.min(SEGMENTS - 1);
        let tick_y = y + h - (seg_idx as f32 + 1.0) * (seg_h + SEG_GAP) + SEG_GAP;

        let tick = Path::rectangle(
            Point::new(x, tick_y),
            Size::new(w, (seg_h * 0.5).max(2.0)),
        );
        frame.fill(&tick, PEAK_COLOR);
    }
}

/// Colour for a segment at normalised position `frac` (0 = bottom, 1 = top).
fn seg_color(frac: f32) -> Color {
    if frac < GREEN_END  { SEG_GREEN  }
    else if frac < YELLOW_END { SEG_YELLOW }
    else                 { SEG_RED    }
}

/// Dim a colour to ~15 % brightness for unlit segments.
fn dim(c: Color) -> Color {
    Color { r: c.r * 0.15, g: c.g * 0.15, b: c.b * 0.15, a: 1.0 }
}
