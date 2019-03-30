#[macro_use]
extern crate simon;
extern crate ines;
extern crate mos6502;

use std::fs::File;
use std::io::{self, Read};

mod debug;
use ines::*;
use mos6502::*;

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

struct NesDevices {
    rom: Vec<u8>,
}

impl MemoryMappedDevices for NesDevices {
    fn read_u8(&mut self, address: Address) -> u8 {
        match address {
            0..=0x7fff => unimplemented!(),
            _ => self.rom[(address as usize - 0x8000) % 0x4000],
        }
    }
    fn write_u8(&mut self, address: Address, _data: u8) {
        unimplemented!()
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
        self.cpu.step(&mut self.devices).unwrap();
    }
}

fn main() {
    let args = Args::arg().with_help_default().parse_env_default_or_exit();
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
            rom: prg_rom.clone(),
        },
    };
    nes.start();
    nes.step();
    nes.step();
    nes.step();
    nes.step();
    println!("{:x?}", nes.cpu);
    debug::print_bytes_hex(&prg_rom, 0xc000, 16);
}
