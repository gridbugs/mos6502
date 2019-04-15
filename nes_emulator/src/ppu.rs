use glutin_frontend::Pixels;

#[derive(Debug)]
pub struct Ppu {
    next_address_write_is_hi_byte: bool,
    address: u16,
    address_increment: u8,
    vblank_nmi: bool,
    sprite_pattern_table_address: u16,
    background_pattern_table_address: u16,
    palette_ram: [u8; 0x20],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            next_address_write_is_hi_byte: true,
            address: 0,
            address_increment: 1,
            vblank_nmi: false,
            sprite_pattern_table_address: 0,
            background_pattern_table_address: 0,
            palette_ram: [0; 0x20],
        }
    }
    pub fn write_control(&mut self, data: u8) {
        if data & control::flag::ADDRESS_INCREMENT != 0 {
            self.address_increment = 32;
        } else {
            self.address_increment = 1;
        }
        self.sprite_pattern_table_address =
            ((data & control::flag::SPRITE_PATTERN_TABLE) != 0) as u16 * 0x1000;
        self.background_pattern_table_address =
            ((data & control::flag::BACKGROUND_PATTERN_TABLE) != 0) as u16 * 0x1000;
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
    pub fn write_data(&mut self, vram: &mut [u8], data: u8) {
        // hardcode horizontal mirroring
        match self.address {
            0x0000..=0x0FFF => println!("unimplemented pattern table write"),
            0x1000..=0x1FFF => println!("unimplemented pattern table write"),
            0x2000..=0x23FF => {
                vram[self.address as usize - 0x2000] = data;
                if self.address >= 0x23C0 {
                    println!("attribute table write {:X} = {:X}", self.address, data);
                }
            }
            0x2400..=0x27FF => vram[self.address as usize - 0x2400] = data,
            0x2800..=0x2BFF => vram[self.address as usize - 0x2400] = data,
            0x2C00..=0x2FFF => vram[self.address as usize - 0x2800] = data,
            0x3000..=0x3EFF => println!("unimplemented mirror write"),
            0x3F00..=0x3F1F => self.palette_ram[self.address as usize - 0x3F00] = data,
            0x3F20..=0x3FFF => println!("unimplemented palette write"),
            _ => panic!("ppu write out of bounds {:X}", self.address),
        }
        self.address = self.address.wrapping_add(self.address_increment as u16);
    }
    pub fn read_data(&mut self, vram: &[u8]) -> u8 {
        panic!()
    }
    pub fn render(&mut self, vram: &[u8], chr_rom: &[u8], mut pixels: Pixels) {
        let nametable = &vram[0x0..=0x3BF];
        let attribute_table = &vram[0x3C0..=0x3FF];
        let universal_background_colour = self.palette_ram[0];
        for (index, &nametable_entry) in nametable.iter().enumerate() {
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
            let palette = &self.palette_ram[palette_base as usize..palette_base as usize + 4];
            let pattern_address = self
                .background_pattern_table_address
                .wrapping_add(nametable_entry as u16 * 16);
            let pattern_lo =
                &chr_rom[pattern_address as usize + 0x0..=pattern_address as usize + 0x7];
            let pattern_hi =
                &chr_rom[pattern_address as usize + 0x8..=pattern_address as usize + 0xF];
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
