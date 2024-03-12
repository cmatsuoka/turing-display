// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;
use std::process;
use std::time;

use rand::Rng;

use clap::Parser;
use turing_screen::framebuffer::Framebuffer;
use turing_screen::{Coord, Image, Rect, Rgba};

use fonts::Font;

mod fonts;

#[derive(Parser)]
#[command(name = "turing-display")]
#[command(about = "A turing smart screen image display utility")]
struct Args {
    /// Serial device to use
    #[arg(short, long, value_name = "device", default_value_t = String::from("AUTO"))]
    port: String,

    /// TTF font to use
    #[arg(short, long, value_name = "font-file")]
    font: String,

    /// Enable debug messages
    #[arg(short, long)]
    debug: bool,

    #[arg(value_name = "png-file")]
    image: String,
}

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(_) => (),
        Err(err) => {
            log::error!("error: {err}");
            process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let level = if args.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level)?;

    log::info!("decoding {}", args.image);
    let mut bitmap = lodepng::decode32_file(args.image)?;
    let mut scr = turing_screen::new("AUTO")?;
    let (width, height) = scr.screen_size();

    log::debug!("framebuffer size: {width}x{height}");
    let mut fb = Framebuffer::new(width, height);

    scr.init()?;
    scr.screen_on()?;
    scr.set_brightness(5)?;

    let background = image_from_bitmap(&mut bitmap);
    let rect = Rect::new(0, 0, background.width, background.height);

    fb.copy_image(&background, &rect, &Coord::new(0, 0));
    fb.render_on(&mut scr, &rect)?;

    let font_data = std::fs::read(args.font)?;
    let font = Font::from_data(font_data)?;

    let mut rng = rand::thread_rng();
    let c1 = Rgba::new(20, 240, 116, 0xff);
    let c2 = Rgba::new(248, 195, 34, 0xff);

    let p1 = Coord::new(23, 309);
    let p2 = Coord::new(23, 80);

    loop {
        fb.copy_image(&background, &rect, &Coord::new(0, 0));

        let v1 = rng.gen_range(0..999);
        let v2 = rng.gen_range(0..999);

        let rect = fonts::draw_text(&mut fb, &font, 110.0, c1, &p1, &format!("{:>3}", v1));
        fb.render_on(&mut scr, &rect)?;

        let rect = fonts::draw_text(&mut fb, &font, 110.0, c2, &p2, &format!("{:>3}", v2));
        fb.render_on(&mut scr, &rect)?;

        std::thread::sleep(time::Duration::from_secs(4));
    }

    scr.screen_off()?;

    Ok(())
}

fn image_from_bitmap(bitmap: &mut lodepng::Bitmap<Rgba>) -> Image {
    Image {
        buffer: &mut bitmap.buffer,
        width: bitmap.width,
        height: bitmap.height,
    }
}
