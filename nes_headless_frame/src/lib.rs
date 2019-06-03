use std::hash::{Hash, Hasher};

pub const NES_SCREEN_WIDTH_PX: u16 = 256;
pub const NES_SCREEN_HEIGHT_PX: u16 = 240;
pub const NES_SCREEN_PX: usize = (NES_SCREEN_WIDTH_PX * NES_SCREEN_HEIGHT_PX) as usize;
pub const NUM_COLOURS: usize = 64;
const COLOUR_MASK: u8 = (NUM_COLOURS as u8) - 1;

mod depth {
    pub const EMPTY: u8 = 0;
    pub const UNIVERSAL_BACKGROUND: u8 = 1;
    pub const SPRITE_BACK: u8 = 2;
    pub const BACKGROUND: u8 = 3;
    pub const SPRITE_FRONT: u8 = 4;
}

pub struct Frame {
    indices: [u8; NES_SCREEN_PX],
    depths: [u8; NES_SCREEN_PX],
}

impl Frame {
    pub fn new() -> Self {
        Self {
            indices: [0; NES_SCREEN_PX],
            depths: [depth::EMPTY; NES_SCREEN_PX],
        }
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn indices(&self) -> &[u8] {
        &self.indices
    }
    fn set_pixel_colour(&mut self, x: u16, y: u16, colour_index: u8, depth: u8) {
        let offset = (y * NES_SCREEN_WIDTH_PX + x) as usize;
        let current_depth = &mut self.depths[offset];
        if depth > *current_depth {
            *current_depth = depth;
            self.indices[offset] = colour_index & COLOUR_MASK;
        }
    }
    pub fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::SPRITE_BACK);
    }
    pub fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::SPRITE_FRONT);
    }
    pub fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::BACKGROUND);
    }
    pub fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::UNIVERSAL_BACKGROUND);
    }
}

impl Hash for Frame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.indices.iter().for_each(|i| i.hash(state));
        self.depths.iter().for_each(|d| d.hash(state));
    }
}
