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
use ppu::Ppu;

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
pub const VRAM_BYTES: usize = 0x800;

struct NesDevices {
    ram: [u8; RAM_BYTES],
    rom: Vec<u8>,
    ppu: Ppu,
    vram: [u8; VRAM_BYTES],
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
                7 => self.ppu.read_data(&self.vram),
                _ => unreachable!(),
            },
            0x4000..=0x7FFF => {
                println!("unimplemented read from {:x}", address);
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
                7 => self.ppu.write_data(&mut self.vram, data),
                _ => unreachable!(),
            },
            0x4000..=0x7FFF => println!("unimplemented write {:x} to {:x}", data, address),
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
        print_bytes_hex(&self.devices.vram, 0, 32);
        print_vram(&self.devices.vram);
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
            vram: [0; VRAM_BYTES],
        },
    };
    nes.start();
    for _ in 0..100000 {
        nes.step();
    }
    nes.nmi();
    for _ in 0..100000 {
        nes.step();
    }
    nes.print_state();
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
        frontend.with_pixels(|pixels| {
            nes.devices
                .ppu
                .render(&mut nes.devices.vram, &chr_rom, pixels)
        });
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
