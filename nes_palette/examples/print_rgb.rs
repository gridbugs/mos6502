use ansi_term::{Colour::RGB, Style};
use nes_palette::{Palette, NUM_COLOURS};

fn main() {
    let palette = Palette::basic();
    for (i, c) in palette.colours().iter().enumerate() {
        println!(
            "{:02x}: {}",
            i,
            Style::new().on(RGB(c.r, c.g, c.b)).paint("     ")
        );
    }
}
