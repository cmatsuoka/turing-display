// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;

use turing_screen::framebuffer::Framebuffer;
use turing_screen::{Coord, Image, Rect, Rgba};

macro_rules! set_min {
    ($a:expr, $b:expr) => {{
        if $a > $b {
            $a = $b;
        }
    }};
}

macro_rules! set_max {
    ($a:expr, $b:expr) => {{
        if $a < $b {
            $a = $b;
        }
    }};
}

pub struct Font<'a> {
    font: rusttype::Font<'a>,
}

impl Font<'_> {
    pub fn from_data(data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        if let Some(font) = rusttype::Font::try_from_vec(data) {
            Ok(Self { font })
        } else {
            Err("cannot load font data".into())
        }
    }
}

pub fn draw_text(
    fb: &mut Framebuffer,
    font: &Font,
    size: f32,
    color: Rgba,
    pos: &Coord,
    msg: &str,
) -> Rect {
    let scale = rusttype::Scale { x: size, y: size };

    // From rusttype ascii.rs:
    // The origin of a line of text is at the baseline (roughly where
    // non-descending letters sit). We don't want to clip the text, so we shift
    // it down with an offset when laying it out. v_metrics.ascent is the
    // distance between the baseline and the highest edge of any glyph in
    // the font. That's enough to guarantee that there's no clipping.
    let v_metrics = font.font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);
    let glyphs: Vec<_> = font.font.layout(msg, scale, offset).collect();

    let h = (v_metrics.ascent + v_metrics.descent).ceil() as usize;
    let w = glyphs // total width of text
        .iter()
        .rev()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    // the text image
    let text_img = &mut Image {
        buffer: &mut vec![turing_screen::TRANSPARENT; w * h],
        width: w,
        height: h,
    };
    // text bounding box in text image coordinates
    let (mut min_x, mut min_y) = (w as i32, h as i32);
    let (mut max_x, mut max_y) = (0i32, 0i32);

    log::debug!("text image size: {}x{}", w, h);

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;

                if v > 0.0 && x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                    let offset = (y * w as i32 + x) as usize;
                    let bg = &mut text_img.buffer[offset];
                    bg.r = color.r;
                    bg.g = color.g;
                    bg.b = color.b;
                    bg.a = (255.0 * v) as u8;
                }
            });
            set_min!(min_x, bb.min.x);
            set_min!(min_y, bb.min.y);
            set_max!(max_x, bb.max.x);
            set_max!(max_y, bb.max.y);
        }
    }

    let (min_x, min_y) = (min_x as usize, min_y as usize);
    let (max_x, max_y) = (max_x as usize, max_y as usize);

    let bb_rect = Rect::new(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1);
    log::debug!("draw text: '{}' {}, bounding box: {}", msg, pos, bb_rect);

    // Blend rasterized text to intermediate buffer
    fb.blend_image(text_img, &bb_rect, pos);

    // The text bounding box in screen coordinates
    Rect::new(pos.x, pos.y, w, h)
}
