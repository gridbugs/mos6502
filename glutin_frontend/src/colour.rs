use nes_palette::{Palette, COLOUR_MASK};

pub struct ColourTable {
    colours: Vec<[f32; 3]>,
}

impl ColourTable {
    pub fn new() -> Self {
        let colours = Palette::basic()
            .to_bytes()
            .chunks(3)
            .map(|c| [c[0] as f32 / 255., c[1] as f32 / 255., c[2] as f32 / 255.])
            .collect();
        Self { colours }
    }
    pub fn lookup(&self, colour_index: u8) -> [f32; 4] {
        let [r, g, b] = self.colours[(colour_index & COLOUR_MASK) as usize];
        [r, g, b, 1.]
    }
}
