#[macro_use]
extern crate simon;
extern crate analyser;
extern crate bincode;
extern crate gif_renderer;
extern crate glutin_frontend;
extern crate ines;
extern crate mos6502;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_big_array;

use glutin_frontend::*;
use ines::Ines;
use mos6502::debug::*;
use mos6502::machine::*;
use mos6502::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

mod ppu;
use ppu::*;

mod apu;
use apu::*;

mod mapper;
use mapper::*;

mod nes;
use nes::*;

#[derive(Debug)]
struct SaveStateArgs {
    frame: u64,
    filename: String,
}

impl SaveStateArgs {
    fn arg() -> simon::ArgExt<impl simon::Arg<Item = Option<Self>>> {
        simon::opt("", "save-state-frame", "save state frame", "INT")
            .depend(simon::opt(
                "",
                "save-state-filename",
                "save state filename",
                "FILE",
            ))
            .option_map(|(frame, filename)| Self { frame, filename })
    }
}

#[derive(Debug)]
struct Args {
    rom_filename: Option<String>,
    state_filename: Option<String>,
    save_state_args: Option<SaveStateArgs>,
}

impl Args {
    fn arg() -> simon::ArgExt<impl simon::Arg<Item = Self>> {
        args_map! {
            let {
                rom_filename = simon::opt("r", "rom", "rom filename", "FILE");
                state_filename = simon::opt("s", "state", "state filename", "FILE");
                save_state_args = SaveStateArgs::arg();
            } in {
                Self {
                    rom_filename,
                    state_filename,
                    save_state_args,
                }
            }
        }
    }
}

struct NesRenderOutput<'a> {
    glutin_pixels: Pixels<'a>,
    gif_frame: &'a mut gif_renderer::Frame,
}

impl<'a> RenderOutput for NesRenderOutput<'a> {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.glutin_pixels
            .set_pixel_colour_sprite_back(x, y, colour_index);
        self.gif_frame
            .set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.glutin_pixels
            .set_pixel_colour_sprite_front(x, y, colour_index);
        self.gif_frame
            .set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.glutin_pixels
            .set_pixel_colour_background(x, y, colour_index);
        self.gif_frame
            .set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.glutin_pixels
            .set_pixel_colour_universal_background(x, y, colour_index);
        self.gif_frame
            .set_pixel_colour_universal_background(x, y, colour_index);
    }
}

#[derive(Serialize, Deserialize)]
pub enum DynamicNes {
    NromHorizontal(Nes<nrom::Nrom<mirroring::Horizontal>>),
    NromVertical(Nes<nrom::Nrom<mirroring::Vertical>>),
    NromFourScreenVram(Nes<nrom::Nrom<mirroring::FourScreenVram>>),
}

#[derive(Debug)]
pub enum Error {
    UnexpectedFormat(mapper::Error),
    InesParseError(ines::Error),
    DeserializeError(bincode::Error),
}

impl DynamicNes {
    fn from_ines(ines: &Ines) -> Result<Self, mapper::Error> {
        let &Ines {
            ref header,
            ref prg_rom,
            ref chr_rom,
        } = ines;
        match (header.mapper, header.mirroring) {
            (ines::Mapper::Nrom, ines::Mirroring::Horizontal) => Ok(DynamicNes::NromHorizontal(
                Nes::new(nrom::Nrom::new(mirroring::Horizontal, &prg_rom, &chr_rom)?),
            )),
            (ines::Mapper::Nrom, ines::Mirroring::Vertical) => Ok(DynamicNes::NromVertical(
                Nes::new(nrom::Nrom::new(mirroring::Vertical, &prg_rom, &chr_rom)?),
            )),
            (ines::Mapper::Nrom, ines::Mirroring::FourScreenVram) => {
                Ok(DynamicNes::NromFourScreenVram(Nes::new(nrom::Nrom::new(
                    mirroring::FourScreenVram,
                    &prg_rom,
                    &chr_rom,
                )?)))
            }
        }
    }
    fn from_args(args: &Args) -> Result<Self, Error> {
        if let Some(ref state_filename) = args.state_filename {
            let mut state_file = File::open(state_filename).expect("Failed to open state file");
            let mut bytes = Vec::new();
            state_file
                .read_to_end(&mut bytes)
                .expect("Failed to read state file");
            bincode::deserialize(&bytes).map_err(Error::DeserializeError)
        } else {
            let buffer = match args.rom_filename.as_ref() {
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
            Self::from_ines(&Ines::parse(&buffer).map_err(Error::InesParseError)?)
                .map_err(Error::UnexpectedFormat)
        }
    }
}

fn save<M: Mapper + serde::Serialize, P: AsRef<Path>>(nes: &Nes<M>, path: P) {
    let bytes = bincode::serialize(&nes.clone_dynamic_nes()).expect("Failed to serialize state");
    let mut file = File::create(path).expect("Failed to create state file");
    file.write_all(&bytes).expect("Failed to write state file");
    println!("Wrote state file");
}

fn run<M: Mapper + serde::Serialize>(mut nes: Nes<M>, save_state_args: Option<SaveStateArgs>) {
    let mut frontend = Frontend::new();
    let mut running = true;
    let mut frame_count = 0;
    let mut output_gif_file = File::create("/tmp/a.gif").unwrap();
    let mut gif_renderer = gif_renderer::Renderer::new(output_gif_file);
    loop {
        if let Some(save_state_args) = save_state_args.as_ref() {
            if frame_count == save_state_args.frame {
                save(&nes, &save_state_args.filename);
            }
        }
        {
            frontend.poll_glutin_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => {
                        running = false;
                    }
                    glutin::WindowEvent::KeyboardInput { input, .. } => match input.state {
                        glutin::ElementState::Pressed => {
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                match virtual_keycode {
                                    glutin::VirtualKeyCode::Left => {
                                        nes::controller1::press::left(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Right => {
                                        nes::controller1::press::right(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Up => {
                                        nes::controller1::press::up(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Down => {
                                        nes::controller1::press::down(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Return => {
                                        nes::controller1::press::start(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::RShift => {
                                        nes::controller1::press::select(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::A => {
                                        nes::controller1::press::a(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::B => {
                                        nes::controller1::press::b(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::S => {
                                        if let Some(save_state_args) = save_state_args.as_ref() {
                                            save(&nes, &save_state_args.filename);
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        glutin::ElementState::Released => {
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                match virtual_keycode {
                                    glutin::VirtualKeyCode::Left => {
                                        nes::controller1::release::left(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Right => {
                                        nes::controller1::release::right(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Up => {
                                        nes::controller1::release::up(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Down => {
                                        nes::controller1::release::down(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::Return => {
                                        nes::controller1::release::start(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::RShift => {
                                        nes::controller1::release::select(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::A => {
                                        nes::controller1::release::a(&mut nes);
                                    }
                                    glutin::VirtualKeyCode::B => {
                                        nes::controller1::release::b(&mut nes);
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            });
        }
        if !running {
            break;
        };
        frontend.with_pixels(|pixels| {
            let mut gif_frame = gif_renderer::Frame::new();
            let mut render_output = NesRenderOutput {
                glutin_pixels: pixels,
                gif_frame: &mut gif_frame,
            };
            nes.render(&mut render_output);
            gif_renderer.add(gif_frame);
        });
        nes.run_for_frame();
        frontend.render();
        frame_count += 1;
    }
}

fn main() {
    let args = Args::arg().with_help_default().parse_env_default_or_exit();
    match DynamicNes::from_args(&args).unwrap() {
        DynamicNes::NromHorizontal(nes) => run(nes, args.save_state_args),
        DynamicNes::NromVertical(nes) => run(nes, args.save_state_args),
        DynamicNes::NromFourScreenVram(nes) => run(nes, args.save_state_args),
    }
}

fn print_bytes_hex(data: &[u8], address_offset: u16, line_width: usize) {
    let stdout = io::stderr();
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
