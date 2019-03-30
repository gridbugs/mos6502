extern crate mos6502;

pub mod nrom {
    use mos6502::*;
    pub const PRG_START_LO: Address = 0x8000;
    pub const PRG_START_HI: Address = 0xc000;
}
