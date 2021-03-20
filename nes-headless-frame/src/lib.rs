use nes_render_output::RenderOutput;
use std::hash::{Hash, Hasher};

mod depth {
    pub const EMPTY: u8 = 0;
    pub const UNIVERSAL_BACKGROUND: u8 = 1;
    pub const SPRITE_BACK: u8 = 2;
    pub const BACKGROUND: u8 = 3;
    pub const SPRITE_FRONT: u8 = 4;
}

#[derive(Clone, PartialEq, Eq)]
pub struct Frame {
    indices: [u8; nes_specs::SCREEN_TOTAL_PX as usize],
    depths: [u8; nes_specs::SCREEN_TOTAL_PX as usize],
    #[cfg(feature = "background_pixel_ages")]
    background_pixel_ages: [u64; nes_specs::SCREEN_TOTAL_PX as usize],
}

impl Frame {
    pub fn new() -> Self {
        Self {
            indices: [0; nes_specs::SCREEN_TOTAL_PX as usize],
            depths: [depth::EMPTY; nes_specs::SCREEN_TOTAL_PX as usize],
            #[cfg(feature = "background_pixel_ages")]
            background_pixel_ages: [0; nes_specs::SCREEN_TOTAL_PX as usize],
        }
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn indices(&self) -> &[u8] {
        &self.indices
    }
    #[cfg(feature = "background_pixel_ages")]
    pub fn background_pixel_ages(&self) -> &[u64] {
        &self.background_pixel_ages
    }
    fn set_pixel_colour(&mut self, x: u16, y: u16, colour_index: u8, depth: u8) {
        let offset = (y * nes_specs::SCREEN_WIDTH_PX + x) as usize;
        let current_depth = &mut self.depths[offset];
        if depth > *current_depth {
            *current_depth = depth;
            self.indices[offset] = colour_index & nes_palette::COLOUR_MASK;
        }
    }
    #[cfg(feature = "background_pixel_ages")]
    pub fn set_background_pixel_age(&mut self, x: u16, y: u16, age: u64) {
        let offset = (y * nes_specs::SCREEN_WIDTH_PX + x) as usize;
        self.background_pixel_ages[offset] = age;
    }
}

impl Hash for Frame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.indices.iter().for_each(|i| i.hash(state));
        self.depths.iter().for_each(|d| d.hash(state));
    }
}

impl RenderOutput for Frame {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::SPRITE_BACK);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::SPRITE_FRONT);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::BACKGROUND);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::UNIVERSAL_BACKGROUND);
    }
}
