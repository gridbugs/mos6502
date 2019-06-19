extern crate rgb24;

pub use rgb24::Rgb24;

pub const NUM_COLOURS: usize = 64;
pub const COLOUR_MASK: u8 = (NUM_COLOURS as u8) - 1;

#[derive(Clone)]
pub struct Palette {
    colours: [Rgb24; NUM_COLOURS],
}

impl Palette {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() != NUM_COLOURS * 3 {
            panic!("unexpected palette length: {}", bytes.len());
        }
        let mut colours = [Rgb24::new(0, 0, 0); NUM_COLOURS];
        bytes
            .chunks(3)
            .map(|chunk| Rgb24::new(chunk[0], chunk[1], chunk[2]))
            .enumerate()
            .for_each(|(i, rgb24)| {
                colours[i] = rgb24;
            });
        Self { colours }
    }
    pub fn basic() -> Self {
        Self::from_bytes(include_bytes!("palette.pal"))
    }
    pub fn to_bytes(&self) -> [u8; NUM_COLOURS * 3] {
        let mut bytes = [0; NUM_COLOURS * 3];
        let mut i = 0;
        for colour in self.colours.iter() {
            bytes[i + 0] = colour.r;
            bytes[i + 1] = colour.g;
            bytes[i + 2] = colour.b;
            i += 3;
        }
        bytes
    }
    pub fn transform<F: FnMut(Rgb24) -> Rgb24>(&mut self, mut f: F) {
        for rgb24 in self.colours.iter_mut() {
            *rgb24 = f(*rgb24);
        }
    }
}
