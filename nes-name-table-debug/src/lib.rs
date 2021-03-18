pub const NAME_TABLE_WIDTH_PX: u16 = nes_specs::SCREEN_WIDTH_PX * 2;
pub const NAME_TABLE_HEIGHT_PX: u16 = nes_specs::SCREEN_HEIGHT_PX * 2;
pub const NAME_TABLE_TOTAL_PX: u32 = NAME_TABLE_WIDTH_PX as u32 * NAME_TABLE_HEIGHT_PX as u32;

#[derive(Clone, Copy)]
pub struct Scroll {
    pub x: u16,
    pub y: u16,
}
pub struct NameTableFrame {
    indices: [u8; NAME_TABLE_TOTAL_PX as usize],
    scroll_by_scanline: [Scroll; nes_specs::SCREEN_HEIGHT_PX as usize],
}

impl NameTableFrame {
    pub fn new() -> Self {
        Self {
            indices: [0; NAME_TABLE_TOTAL_PX as usize],
            scroll_by_scanline: [Scroll { x: 0, y: 0 }; nes_specs::SCREEN_HEIGHT_PX as usize],
        }
    }
    pub fn set_pixel(&mut self, x: u16, y: u16, index: u8) {
        self.indices[(y as u32 * NAME_TABLE_WIDTH_PX as u32 + x as u32) as usize] = index;
    }
    pub fn set_scroll(&mut self, scanline: u8, x: u16, y: u16) {
        self.scroll_by_scanline[scanline as usize] = Scroll { x, y };
    }
    pub fn indices(&self) -> [u8; NAME_TABLE_TOTAL_PX as usize] {
        let mut indices = self.indices;
        for Scroll { x, y } in self.scroll_by_scanline.iter() {
            let pixel_y = *y;
            if pixel_y < NAME_TABLE_HEIGHT_PX {
                for i in 0..nes_specs::SCREEN_WIDTH_PX {
                    let mut pixel_x = x + i;
                    if pixel_x >= NAME_TABLE_WIDTH_PX {
                        pixel_x -= NAME_TABLE_WIDTH_PX;
                    }
                    let pixel_index =
                        pixel_y as usize * NAME_TABLE_WIDTH_PX as usize + pixel_x as usize;
                    if indices[pixel_index] < nes_palette::NUM_COLOURS as u8 {
                        indices[pixel_index] += nes_palette::NUM_COLOURS as u8;
                    }
                }
            }
        }
        indices
    }
}
