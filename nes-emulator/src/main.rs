#[macro_use]
extern crate simon;

use std::fs::File;
use std::io::{self, Read};

mod debug;
mod ines;
use ines::Ines;

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
    debug::print_bytes_hex(&prg_rom, 16);
    /*
    let pclo = prg_rom[0xfffc - 0xc000] as u16;
    let pchi = prg_rom[0xfffd - 0xc000] as u16;
    let pc = (pchi << 8 | pclo) as usize;
    let inst = Instruction::decode(prg_rom[pc - 0xc000]);
    println!("{:?}", inst);*/
}
