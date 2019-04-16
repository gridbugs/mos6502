use glutin_frontend::Pixels;

#[derive(Debug)]
pub struct Ppu {
    next_address_write_is_hi_byte: bool,
    address: u16,
    address_increment: u8,
    vblank_nmi: bool,
    sprite_pattern_table: PatternTableChoice,
    background_pattern_table: PatternTableChoice,
}

pub type PpuAddress = u16;
pub const PATTERN_TABLE_BYTES: usize = 0x1000;
pub const NAME_TABLE_BYTES: usize = 0x400;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PatternTableChoice {
    PatternTable0,
    PatternTable1,
}

impl PatternTableChoice {
    pub fn base_address(self) -> PpuAddress {
        self as PpuAddress * (PATTERN_TABLE_BYTES as PpuAddress)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum NameTableChoice {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl NameTableChoice {
    pub fn address_offset_horizontal_mirror(self) -> PpuAddress {
        (self as PpuAddress / 2) * (NAME_TABLE_BYTES as PpuAddress)
    }
}

pub trait PpuMemory {
    fn write_u8(&mut self, address: PpuAddress, data: u8);
    fn read_u8(&self, address: PpuAddress) -> u8;
    fn pattern_table(&self, choice: PatternTableChoice) -> &[u8];
    fn name_table(&self, choice: NameTableChoice) -> &[u8];
    fn palette_ram(&self) -> &[u8];
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            next_address_write_is_hi_byte: true,
            address: 0,
            address_increment: 1,
            vblank_nmi: false,
            sprite_pattern_table: PatternTableChoice::PatternTable0,
            background_pattern_table: PatternTableChoice::PatternTable0,
        }
    }
    pub fn write_control(&mut self, data: u8) {
        self.address_increment = if data & control::flag::ADDRESS_INCREMENT != 0 {
            32
        } else {
            1
        };
        self.sprite_pattern_table = if (data & control::flag::SPRITE_PATTERN_TABLE) == 0 {
            PatternTableChoice::PatternTable0
        } else {
            PatternTableChoice::PatternTable1
        };
        self.background_pattern_table = if (data & control::flag::BACKGROUND_PATTERN_TABLE) == 0 {
            PatternTableChoice::PatternTable0
        } else {
            PatternTableChoice::PatternTable1
        };
        self.vblank_nmi = data & control::flag::VBLANK_NMI != 0;
    }
    pub fn write_mask(&mut self, _data: u8) {}
    pub fn read_status(&mut self) -> u8 {
        self.address = 0;
        self.next_address_write_is_hi_byte = true;
        status::flag::VBLANK
    }
    pub fn write_oam_address(&mut self, _data: u8) {}
    pub fn write_oam_data(&mut self, _data: u8) {}
    pub fn read_oam_data(&mut self) -> u8 {
        0
    }
    pub fn write_scroll(&mut self, _data: u8) {}
    pub fn write_address(&mut self, data: u8) {
        self.address |= (data as u16).wrapping_shl((self.next_address_write_is_hi_byte as u32) * 8);
        self.next_address_write_is_hi_byte = !self.next_address_write_is_hi_byte;
    }
    pub fn write_data<M: PpuMemory>(&mut self, memory: &mut M, data: u8) {
        memory.write_u8(self.address, data);
        self.address = self.address.wrapping_add(self.address_increment as u16);
    }
    pub fn read_data<M: PpuMemory>(&mut self, memory: &M) -> u8 {
        let data = memory.read_u8(self.address);
        self.address = self.address.wrapping_add(self.address_increment as u16);
        data
    }
    pub fn render<M: PpuMemory>(&mut self, memory: &M, mut pixels: Pixels) {
        let name_table_and_attribute_table = memory.name_table(NameTableChoice::TopLeft);
        let name_table = &name_table_and_attribute_table[0x0..=0x3BF];
        let attribute_table = &name_table_and_attribute_table[0x3C0..=0x3FF];
        let palette_ram = memory.palette_ram();
        let universal_background_colour = palette_ram[0];
        let background_pattern_table = memory.pattern_table(self.background_pattern_table);
        for (index, &name_table_entry) in name_table.iter().enumerate() {
            let x = index % 32;
            let y = index / 32;
            let attribute_base = (y / 4) * 8 + (x / 4);
            let attribute = attribute_table[attribute_base as usize];
            let top = y & 2 == 0;
            let left = x & 2 == 0;
            let shift = match (top, left) {
                (true, true) => 0,
                (true, false) => 2,
                (false, true) => 4,
                (false, false) => 6,
            };
            let palette_base = (attribute.wrapping_shr(shift) & 0x3) * 4;
            let palette = &palette_ram[palette_base as usize..palette_base as usize + 4];
            let pattern_address = name_table_entry as u16 * 16;
            let pattern_lo = &background_pattern_table
                [pattern_address as usize + 0x0..=pattern_address as usize + 0x7];
            let pattern_hi = &background_pattern_table
                [pattern_address as usize + 0x8..=pattern_address as usize + 0xF];
            for (row_index, (&pixel_row_lo, &pixel_row_hi)) in
                pattern_lo.iter().zip(pattern_hi.iter()).enumerate()
            {
                for i in 0..8 {
                    let palette_index_lo = pixel_row_lo & 128u8.wrapping_shr(i) != 0;
                    let palette_index_hi = pixel_row_hi & 128u8.wrapping_shr(i) != 0;
                    let palette_index =
                        palette_index_lo as u8 | (palette_index_hi as u8).wrapping_shl(1);
                    let colour_code = match palette_index {
                        0 => universal_background_colour,
                        _ => palette[palette_index as usize],
                    };
                    pixels.set_pixel_colour(
                        x as u16 * 8 + i as u16,
                        y as u16 * 8 + row_index as u16,
                        colour_code,
                    );
                }
            }
        }
    }
}

pub mod control {
    pub mod bit {
        pub const VBLANK_NMI: u8 = 7;
        pub const ADDRESS_INCREMENT: u8 = 2;
        pub const SPRITE_PATTERN_TABLE: u8 = 3;
        pub const BACKGROUND_PATTERN_TABLE: u8 = 4;
    }
    pub mod flag {
        use super::bit;
        pub const VBLANK_NMI: u8 = 1 << bit::VBLANK_NMI;
        pub const ADDRESS_INCREMENT: u8 = 1 << bit::ADDRESS_INCREMENT;
        pub const SPRITE_PATTERN_TABLE: u8 = 1 << bit::SPRITE_PATTERN_TABLE;
        pub const BACKGROUND_PATTERN_TABLE: u8 = 1 << bit::BACKGROUND_PATTERN_TABLE;
    }
}

pub mod status {
    pub mod bit {
        pub const VBLANK: u8 = 7;
    }
    pub mod flag {
        use super::bit;
        pub const VBLANK: u8 = 1 << bit::VBLANK;
    }
}
