#[macro_use]
extern crate simon;
extern crate glutin_frontend;
extern crate ines;
extern crate mos6502;

use glutin_frontend::*;
use ines::*;
use mos6502::debug::*;
use mos6502::machine::*;
use mos6502::*;
use std::fs::File;
use std::io::{self, Read, Write};

mod ppu;
use ppu::*;

#[derive(Debug)]
struct Args {
    rom_filename: Option<String>,
}

impl Args {
    fn arg() -> simon::ArgExt<impl simon::Arg<Item = Self>> {
        args_map! {
            let {
                rom_filename = simon::opt("r", "rom", "rom filename", "FILE");
            } in {
                Self { rom_filename }
            }
        }
    }
}

const RAM_BYTES: usize = 0x800;
const PALETTE_RAM_BYTES: usize = 0x20;
const NAME_TABLE_RAM_BYTES: usize = NAME_TABLE_BYTES * 2;

struct NesDevices {
    ram: [u8; RAM_BYTES],
    rom: Vec<u8>,
    ppu: Ppu,
    ppu_memory: NesPpuMemory,
}

struct NesPpuMemory {
    name_table_ram: [u8; NAME_TABLE_RAM_BYTES],
    chr_rom: Vec<u8>,
    palette_ram: [u8; PALETTE_RAM_BYTES],
}

impl PpuMemory for NesPpuMemory {
    fn write_u8(&mut self, address: PpuAddress, data: u8) {
        match address % 0x4000 {
            0x0000..=0x0FFF => panic!("unimplemented pattern table write"),
            0x1000..=0x1FFF => panic!("unimplemented pattern table write"),
            0x2000..=0x23FF => self.name_table_ram[address as usize - 0x2000] = data,
            0x2400..=0x27FF => self.name_table_ram[address as usize - 0x2400] = data,
            0x2800..=0x2BFF => self.name_table_ram[address as usize - 0x2400] = data,
            0x2C00..=0x2FFF => self.name_table_ram[address as usize - 0x2800] = data,
            0x3000..=0x33FF => self.name_table_ram[address as usize - 0x3000] = data,
            0x3400..=0x37FF => self.name_table_ram[address as usize - 0x3400] = data,
            0x3800..=0x3BFF => self.name_table_ram[address as usize - 0x3400] = data,
            0x3C00..=0x3EFF => self.name_table_ram[address as usize - 0x3800] = data,
            0x3F00..=0x3F1F => self.palette_ram[address as usize - 0x3F00] = data,
            0x3F20..=0x3FFF => self.palette_ram[(address as usize - 0x3F20) % 0x20] = data,
            _ => unreachable!(),
        }
    }
    fn read_u8(&self, address: PpuAddress) -> u8 {
        unimplemented!()
    }
    fn pattern_table(&self, choice: PatternTableChoice) -> &[u8] {
        let base_address = choice.base_address() as usize;
        &self.chr_rom[base_address..(base_address + PATTERN_TABLE_BYTES)]
    }
    fn name_table(&self, choice: NameTableChoice) -> &[u8] {
        let address_offset = choice.address_offset_horizontal_mirror() as usize;
        &self.name_table_ram[address_offset..(address_offset + NAME_TABLE_BYTES)]
    }
    fn palette_ram(&self) -> &[u8] {
        &self.palette_ram
    }
}

impl Memory for NesDevices {
    fn read_u8(&mut self, address: Address) -> u8 {
        match address {
            0..=0x1FFF => self.ram[address as usize % RAM_BYTES],
            0x2000..=0x3FFF => match address % 8 {
                0 => 0,
                1 => 0,
                2 => self.ppu.read_status(),
                3 => 0,
                4 => self.ppu.read_oam_data(),
                5 => 0,
                6 => 0,
                7 => self.ppu.read_data(&self.ppu_memory),
                _ => unreachable!(),
            },
            0x4000..=0x7FFF => {
                //println!("unimplemented read from {:x}", address);
                0
            }
            0x8000..=0xFFFF => self.rom[(address as usize - 0x8000) % 0x4000],
        }
    }
    fn write_u8(&mut self, address: Address, data: u8) {
        match address {
            0..=0x1FFF => self.ram[address as usize % RAM_BYTES] = data,
            0x2000..=0x3FFF => match address % 8 {
                0 => self.ppu.write_control(data),
                1 => self.ppu.write_mask(data),
                2 => (),
                3 => self.ppu.write_oam_address(data),
                4 => self.ppu.write_oam_data(data),
                5 => self.ppu.write_scroll(data),
                6 => self.ppu.write_address(data),
                7 => self.ppu.write_data(&mut self.ppu_memory, data),
                _ => unreachable!(),
            },
            0x4000..=0x7FFF => (), //println!("unimplemented write {:x} to {:x}", data, address),
            0x8000..=0xFFFF => panic!("unimplemented write {:x} to {:x}", data, address),
        }
    }
}

impl MemoryReadOnly for NesDevices {
    fn read_u8_read_only(&self, address: Address) -> u8 {
        match address {
            0..=0x7FFF => panic!("unimplemented read from {:x}", address),
            0x8000..=0xFFFF => self.rom[(address as usize - 0x8000) % 0x4000],
        }
    }
}

struct Nes {
    cpu: Cpu,
    devices: NesDevices,
}

impl Nes {
    fn start(&mut self) {
        self.cpu.start(&mut self.devices);
    }
    fn step(&mut self) {
        let instruction_with_operand =
            InstructionWithOperand::next(&self.cpu, &self.devices).unwrap();
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let _ = writeln!(handle, "{}", instruction_with_operand);
        match self.cpu.step(&mut self.devices) {
            Ok(_) => (),
            Err(UnknownOpcode(opcode)) => {
                self.print_state();
                panic!("Unknown opcode: {:x} ({:x?})", opcode, self.cpu);
            }
        }
    }
    fn run_for_cycles(&mut self, num_cycles: usize) {
        self.cpu
            .run_for_cycles(&mut self.devices, num_cycles)
            .unwrap();
    }
    fn nmi(&mut self) {
        self.cpu.nmi(&mut self.devices);
    }
    fn print_state(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let _ = writeln!(handle, "CPU");
        let _ = writeln!(handle, "{:X?}", self.cpu);
        let _ = writeln!(handle, "\nROM");
        print_bytes_hex(&self.devices.rom, 0xC000, 16);
        let _ = writeln!(handle, "\nRAM");
        print_bytes_hex(&self.devices.ram, 0, 16);
        let _ = writeln!(handle, "\nVRAM");
        print_bytes_hex(&self.devices.ppu_memory.name_table_ram, 0, 32);
        print_vram(&self.devices.ppu_memory.name_table_ram);
        let _ = writeln!(handle, "PPU");
        let _ = writeln!(handle, "{:X?}", self.devices.ppu);
    }
}

fn main() {
    let args = Args::arg().with_help_default().parse_env_default_or_exit();
    let mut frontend = Frontend::new();
    let buffer = match args.rom_filename {
        Some(rom_filename) => {
            let mut buffer = Vec::new();
            let mut rom_file = File::open(rom_filename).expect("Failed to open rom file");
            rom_file
                .read_to_end(&mut buffer)
                .expect("Failed to read rom file");
            buffer
        }
        None => {
            let mut buffer = Vec::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle
                .read_to_end(&mut buffer)
                .expect("Failed to read rom from stdin");
            buffer
        }
    };
    let Ines {
        prg_rom, chr_rom, ..
    } = Ines::parse(&buffer);
    let mut nes = Nes {
        cpu: Cpu::new(),
        devices: NesDevices {
            ram: [0; RAM_BYTES],
            rom: prg_rom.clone(),
            ppu: Ppu::new(),
            ppu_memory: NesPpuMemory {
                name_table_ram: [0; NAME_TABLE_RAM_BYTES],
                chr_rom: chr_rom.clone(),
                palette_ram: [0; PALETTE_RAM_BYTES],
            },
        },
    };
    nes.start();
    let mut running = true;
    loop {
        frontend.poll_glutin_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => {
                    running = false;
                }
                _ => (),
            },
            _ => (),
        });
        if !running {
            break;
        }
        frontend.with_pixels(|pixels| nes.devices.ppu.render(&nes.devices.ppu_memory, pixels));
        nes.run_for_cycles(25000);
        nes.nmi();
        frontend.render();
    }
}

pub fn print_bytes_hex(data: &[u8], address_offset: u16, line_width: usize) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for (i, chunk) in data.chunks(line_width).enumerate() {
        let _ = write!(handle, "{:04X}: ", address_offset as usize + i * line_width);
        for x in chunk {
            let _ = write!(handle, "{:02X}  ", x);
        }
        let _ = writeln!(handle, "");
    }
}

pub fn print_vram(data: &[u8]) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for (i, chunk) in data.chunks(32).enumerate() {
        for x in chunk {
            let c = match x {
                0x24 => ' ',
                0x62 => '#',
                _ => '.',
            };
            let _ = write!(handle, "{}", c);
        }
        let _ = writeln!(handle, "");
    }
}
