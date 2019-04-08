#[derive(Debug)]
pub struct Ppu {
    address_write_offset: u8,
    address: u16,
    address_increment: u8,
    vblank_nmi: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            address_write_offset: 8,
            address: 0,
            address_increment: 1,
            vblank_nmi: false,
        }
    }
    pub fn write_control(&mut self, data: u8) {
        if data & control::flag::ADDRESS_INCREMENT != 0 {
            self.address_increment = 32;
        } else {
            self.address_increment = 1;
        }
        self.vblank_nmi = data & control::flag::VBLANK_NMI != 0;
    }
    pub fn write_mask(&mut self, _data: u8) {}
    pub fn read_status(&mut self) -> u8 {
        self.address = 0;
        self.address_write_offset = 8;
        status::flag::VBLANK
    }
    pub fn write_oam_address(&mut self, _data: u8) {}
    pub fn write_oam_data(&mut self, _data: u8) {}
    pub fn read_oam_data(&mut self) -> u8 {
        0
    }
    pub fn write_scroll(&mut self, _data: u8) {}
    pub fn write_address(&mut self, data: u8) {
        self.address |= (data as u16).wrapping_shl(self.address_write_offset as u32);
        println!("write address {:X} {:X}", data, self.address);
        self.address_write_offset = 0;
    }
    pub fn write_data(&mut self, vram: &mut [u8], data: u8) {
        println!("{:X}", self.address);
        // hardcode horizontal mirroring
        match self.address {
            0x0000..=0x0FFF => println!("unimplemented pattern table write"),
            0x1000..=0x1FFF => println!("unimplemented pattern table write"),
            0x2000..=0x23FF => vram[self.address as usize - 0x2000] = data,
            0x2400..=0x27FF => vram[self.address as usize - 0x2400] = data,
            0x2800..=0x2BFF => vram[self.address as usize - 0x2400] = data,
            0x2C00..=0x2FFF => vram[self.address as usize - 0x2800] = data,
            0x3000..=0x3EFF => println!("unimplemented mirror write"),
            0x3F00..=0x3FFF => println!("unimplemented palette write"),
            _ => panic!("ppu write out of bounds"),
        }
        self.address = self.address.wrapping_add(self.address_increment as u16);
    }
    pub fn read_data(&mut self, vram: &[u8]) -> u8 {
        panic!()
    }
}

pub mod control {
    pub mod bit {
        pub const VBLANK_NMI: u8 = 7;
        pub const ADDRESS_INCREMENT: u8 = 2;
    }
    pub mod flag {
        use super::bit;
        pub const VBLANK_NMI: u8 = 1 << bit::VBLANK_NMI;
        pub const ADDRESS_INCREMENT: u8 = 1 << bit::ADDRESS_INCREMENT;
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
