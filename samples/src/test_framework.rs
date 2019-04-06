use crate::*;
use mos6502::interrupt_vector;
use mos6502::machine::*;

const RAM_BYTES: usize = 0x800;
const ROM_BYTES: usize = 0x4000;

struct Devices {
    ram: [u8; RAM_BYTES],
    rom: Vec<u8>,
}

impl MemoryReadOnly for Devices {
    fn read_u8_read_only(&self, address: Address) -> u8 {
        match address {
            0..=0x7FF => self.ram[address as usize],
            0x800..=0xBFFF => panic!("Unexpected read from {:X}", address),
            PRG_START..=0xFFFF => self.rom[address as usize - PRG_START as usize],
        }
    }
}

impl Memory for Devices {
    fn read_u8(&mut self, address: Address) -> u8 {
        self.read_u8_read_only(address)
    }
    fn write_u8(&mut self, address: Address, data: u8) {
        match address {
            0..=0x7FF => self.ram[address as usize] = data,
            0x800..=0xFFFF => panic!("Unexpected write of {:X} to {:X}", data, address),
        }
    }
}

const INTERRUPT_VECTOR_START_PC_OFFSET: Address = interrupt_vector::START_PC_LO - PRG_START;

pub fn test_sample<S: Sample>(_: S) {
    let mut block = Block::new();
    S::program(&mut block);
    block.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
    block.literal_offset_le(0);
    let mut rom = Vec::new();
    block
        .assemble(PRG_START, ROM_BYTES, &mut rom)
        .expect("Failed to assemble");
    let mut devices = Devices {
        ram: [0; RAM_BYTES],
        rom,
    };
    let mut cpu = Cpu::new();
    cpu.start(&mut devices);
    for _ in 0..S::num_steps() {
        cpu.step(&mut devices).unwrap();
    }
    S::check_result(&cpu, &devices);
}
