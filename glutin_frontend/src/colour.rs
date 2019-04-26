const NUM_COLOURS: usize = 64;
const COLOUR_MASK: u8 = (NUM_COLOURS as u8) - 1;

pub struct ColourTable {
    colours: Vec<[f32; 3]>,
}

impl ColourTable {
    pub fn new() -> Self {
        let bytes = include_bytes!("palette.pal");
        let colours = bytes
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
