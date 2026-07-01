// TCNJ AI/ML Group
//
// Renders a Bible verse (reference + body text) into an RGBA pixel buffer for NDI output.
// Uses fontdue for pure-Rust font rasterization with Windows system fonts.

use fontdue::{Font, FontSettings};

pub struct FrameRenderer {
    font: Font,
}

impl FrameRenderer {
    /// Try to load a system font and return a renderer, or `None` if no font is available.
    pub fn new() -> Option<Self> {
        let candidates = [
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\calibri.ttf",
            r"C:\Windows\Fonts\arial.ttf",
        ];
        for path in &candidates {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(font) = Font::from_bytes(data.as_slice(), FontSettings::default()) {
                    return Some(Self { font });
                }
            }
        }
        None
    }

    /// Render `reference` and `verse_text` centred on a `width × height` black RGBA canvas.
    pub fn render_verse(&self, reference: &str, verse_text: &str, width: u32, height: u32) -> Vec<u8> {
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        // Black opaque background
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[3] = 255;
        }

        if reference.is_empty() && verse_text.is_empty() {
            return pixels;
        }

        // Font sizes proportional to canvas height
        let ref_px = (height as f32 * 0.050).clamp(20.0, 72.0);
        let body_px = (height as f32 * 0.033).clamp(14.0, 48.0);

        let ref_lm = self.font.horizontal_line_metrics(ref_px);
        let body_lm = self.font.horizontal_line_metrics(body_px);

        let ref_line_h = ref_lm.map(|m| m.new_line_size).unwrap_or(ref_px * 1.2);
        let body_line_h = body_lm.map(|m| m.new_line_size).unwrap_or(body_px * 1.35);

        let max_body_w = (width as f32 * 0.80) as u32;
        let wrapped = self.wrap_text(verse_text, body_px, max_body_w);

        // Vertical: centre the whole block
        let gap = ref_px * 0.55;
        let block_h = ref_line_h + gap + wrapped.len() as f32 * body_line_h;
        let block_top = ((height as f32 - block_h) / 2.0).max(16.0);

        // Draw reference in bright white
        let ref_ascent = ref_lm.map(|m| m.ascent).unwrap_or(ref_px * 0.80);
        self.draw_line_centered(
            &mut pixels, width, height,
            reference, ref_px,
            block_top + ref_ascent,
            [255, 255, 255, 255],
        );

        // Draw verse body in off-white
        let body_ascent = body_lm.map(|m| m.ascent).unwrap_or(body_px * 0.80);
        let body_top = block_top + ref_line_h + gap;
        for (i, line) in wrapped.iter().enumerate() {
            let baseline = body_top + body_ascent + i as f32 * body_line_h;
            self.draw_line_centered(
                &mut pixels, width, height,
                line, body_px, baseline,
                [210, 210, 210, 255],
            );
        }

        pixels
    }

    // ── private helpers ──────────────────────────────────────────────────────

    fn measure_width(&self, text: &str, px: f32) -> f32 {
        text.chars().map(|c| self.font.metrics(c, px).advance_width).sum()
    }

    fn wrap_text(&self, text: &str, px: f32, max_w: u32) -> Vec<String> {
        let space_w = self.font.metrics(' ', px).advance_width;
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut cur_w = 0.0f32;

        for word in text.split_whitespace() {
            let word_w: f32 = word.chars().map(|c| self.font.metrics(c, px).advance_width).sum();
            if cur.is_empty() {
                cur.push_str(word);
                cur_w = word_w;
            } else if cur_w + space_w + word_w <= max_w as f32 {
                cur.push(' ');
                cur.push_str(word);
                cur_w += space_w + word_w;
            } else {
                lines.push(std::mem::take(&mut cur));
                cur.push_str(word);
                cur_w = word_w;
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    fn draw_line_centered(
        &self,
        pixels: &mut [u8],
        canvas_w: u32,
        canvas_h: u32,
        text: &str,
        px: f32,
        baseline_y: f32,
        color: [u8; 4],
    ) {
        let line_w = self.measure_width(text, px);
        let start_x = ((canvas_w as f32 - line_w) / 2.0).max(0.0);
        let mut cursor_x = start_x;

        for ch in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(ch, px);
            if metrics.width == 0 || metrics.height == 0 {
                cursor_x += metrics.advance_width;
                continue;
            }

            // fontdue uses y-up; convert to screen y-down:
            // glyph top in screen space = baseline_y - (ymin + height)
            let glyph_top = baseline_y - (metrics.ymin as f32 + metrics.height as f32);
            let glyph_left = cursor_x + metrics.xmin as f32;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let coverage = bitmap[row * metrics.width + col];
                    if coverage == 0 {
                        continue;
                    }
                    let px_x = (glyph_left + col as f32) as i32;
                    let px_y = (glyph_top + row as f32) as i32;
                    if px_x < 0 || px_y < 0
                        || px_x >= canvas_w as i32
                        || px_y >= canvas_h as i32
                    {
                        continue;
                    }
                    let idx = (px_y as u32 * canvas_w + px_x as u32) as usize * 4;
                    let a = coverage as f32 / 255.0;
                    pixels[idx]     = lerp(color[0], pixels[idx],     a);
                    pixels[idx + 1] = lerp(color[1], pixels[idx + 1], a);
                    pixels[idx + 2] = lerp(color[2], pixels[idx + 2], a);
                    // alpha stays 255 (opaque)
                }
            }

            cursor_x += metrics.advance_width;
        }
    }
}

#[inline]
fn lerp(src: u8, dst: u8, t: f32) -> u8 {
    (src as f32 * t + dst as f32 * (1.0 - t)) as u8
}
