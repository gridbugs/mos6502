pub struct Ppu {}

impl Ppu {
    pub fn new() -> Self {
        Self {}
    }
    pub fn write_control(&mut self, _data: u8) {}
    pub fn write_mask(&mut self, _data: u8) {}
    pub fn read_status(&mut self) -> u8 {
        0
    }
    pub fn write_oam_address(&mut self, _data: u8) {}
    pub fn write_oam_data(&mut self, _data: u8) {}
    pub fn read_oam_data(&mut self) -> u8 {
        0
    }
    pub fn write_scroll(&mut self, _data: u8) {}
    pub fn write_address(&mut self, _data: u8) {}
    pub fn write_data(&mut self, _data: u8) {}
    pub fn read_data(&mut self) -> u8 {
        0
    }
}
