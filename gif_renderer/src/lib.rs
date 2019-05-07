extern crate gif;
use gif::SetParameter;
use std::borrow::Cow;
use std::io;

const NES_SCREEN_WIDTH_PX: u16 = 256;
const NES_SCREEN_HEIGHT_PX: u16 = 240;
const NES_SCREEN_PX: usize = (NES_SCREEN_WIDTH_PX * NES_SCREEN_HEIGHT_PX) as usize;
const NUM_COLOURS: usize = 64;
const COLOUR_MASK: u8 = (NUM_COLOURS as u8) - 1;

pub struct Frame {
    indices: [u8; NES_SCREEN_PX],
}

impl Frame {
    pub fn new() -> Self {
        Self {
            indices: [0; NES_SCREEN_PX],
        }
    }
    pub fn set_pixel_colour(&mut self, x: u16, y: u16, colour_index: u8) {
        self.indices[(y * NES_SCREEN_WIDTH_PX + x) as usize] = colour_index & COLOUR_MASK;
    }
}

pub struct Renderer<W: io::Write> {
    encoder: gif::Encoder<W>,
}

impl<W: io::Write> Renderer<W> {
    pub fn new(output: W) -> Self {
        let palette = include_bytes!("palette.pal");
        if palette.len() != NUM_COLOURS * 3 {
            panic!("unexpected palette length: {}", palette.len());
        }
        let mut encoder =
            gif::Encoder::new(output, NES_SCREEN_WIDTH_PX, NES_SCREEN_HEIGHT_PX, palette).unwrap();
        encoder.set(gif::Repeat::Infinite).unwrap();
        Self { encoder }
    }
    pub fn add(&mut self, frame: Frame) {
        let mut gif_frame = gif::Frame::default();
        gif_frame.delay = 3; // 30 ms
        gif_frame.width = NES_SCREEN_WIDTH_PX;
        gif_frame.height = NES_SCREEN_HEIGHT_PX;
        gif_frame.buffer = Cow::Borrowed(&frame.indices);
        self.encoder.write_frame(&gif_frame).unwrap();
    }
}
