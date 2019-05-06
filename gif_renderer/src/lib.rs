extern crate gif;
use std::io;

pub const NES_SCREEN_WIDTH_PX: u16 = 256;
pub const NES_SCREEN_HEIGHT_PX: u16 = 240;
const NES_SCREEN_PX: usize = (NES_SCREEN_WIDTH_PX * NES_SCREEN_HEIGHT_PX) as usize;

pub struct Frame {
    indices: [u8; NES_SCREEN_PX],
}

impl Frame {
    pub fn new() -> Self {
        Self {
            indices: [0; NES_SCREEN_PX],
        }
    }
}

pub struct Renderer {
    frames: Vec<Frame>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }
    pub fn add(&mut self, frame: Frame) {
        self.frames.push(frame);
    }
    pub fn encode<W: io::Write>(&self, w: W) -> io::Result<()> {
        let palette = include_bytes!("palette.pal");
        Ok(())
    }
}
