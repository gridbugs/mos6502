extern crate gif;
extern crate nes_headless_frame;
use gif::SetParameter;
use std::borrow::Cow;
use std::io;

pub use nes_headless_frame::Frame;

pub struct Renderer<W: io::Write> {
    encoder: gif::Encoder<W>,
}

impl<W: io::Write> Renderer<W> {
    pub fn new(output: W) -> Self {
        let palette = include_bytes!("palette.pal");
        if palette.len() != nes_headless_frame::NUM_COLOURS * 3 {
            panic!("unexpected palette length: {}", palette.len());
        }
        let mut encoder = gif::Encoder::new(
            output,
            nes_headless_frame::NES_SCREEN_WIDTH_PX,
            nes_headless_frame::NES_SCREEN_HEIGHT_PX,
            palette,
        )
        .unwrap();
        encoder.set(gif::Repeat::Infinite).unwrap();
        Self { encoder }
    }
    pub fn add(&mut self, frame: Frame) {
        let mut gif_frame = gif::Frame::default();
        gif_frame.delay = 3;
        gif_frame.width = nes_headless_frame::NES_SCREEN_WIDTH_PX;
        gif_frame.height = nes_headless_frame::NES_SCREEN_HEIGHT_PX;
        gif_frame.buffer = Cow::Borrowed(frame.indices());
        self.encoder.write_frame(&gif_frame).unwrap();
    }
}
