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
extern crate nes_headless_frame;

mod apu;
mod mapper;
mod nes;
mod ppu;

use glutin_frontend::glutin;
use ines::Ines;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use mapper::{mmc1, nrom, Mapper};
use nes::Nes;
use ppu::RenderOutput;

enum Frontend {
    Glutin(glutin_frontend::Frontend),
    HeadlessPrintingFinalFrameHash { num_frames: u64 },
}

impl Frontend {
    fn arg() -> simon::ArgExt<impl simon::Arg<Item = Self>> {
        simon::opt(
            "e",
            "headless-num-frames",
            "run with headless frontend, exiting after a specified number of frames",
            "INT",
        )
        .option_map(|num_frames| Frontend::HeadlessPrintingFinalFrameHash { num_frames })
        .with_default_lazy(|| Frontend::Glutin(glutin_frontend::Frontend::new()))
    }
}

#[derive(Clone)]
enum Input {
    Stdin,
    RomFile(String),
    StateFile(String),
}

impl Input {
    fn arg() -> simon::ArgExt<impl simon::Arg<Item = Self>> {
        let rom_file = simon::opt("r", "rom-file", "rom file (ines format) to load", "PATH")
            .option_map(Input::RomFile);
        let load_state_file = simon::opt("l", "load-state-file", "state file to load", "PATH")
            .option_map(Input::StateFile);
        rom_file.either(load_state_file).with_default(Input::Stdin)
    }
}

struct Args {
    input: Input,
    autosave_after_frames: Option<u64>,
    kill_after_frames: Option<u64>,
    save_state_filename: Option<String>,
    gif_filename: Option<String>,
    frontend: Frontend,
}

impl Args {
    fn arg() -> simon::ArgExt<impl simon::Arg<Item = Self>> {
        args_map! {
            let {
                input = Input::arg();
                autosave_after_frames = simon::opt("a", "autosave-after-frames", "save state after this many frames", "INT");
                kill_after_frames = simon::opt("k", "kill-after-frames", "exit after this many frames", "INT");
                save_state_filename = simon::opt("s", "save-state-file", "state file to save", "PATH");
                gif_filename = simon::opt("g", "gif", "gif file to record into", "PATH");
                frontend = Frontend::arg();
            } in {
                Self {
                    input,
                    autosave_after_frames,
                    kill_after_frames,
                    save_state_filename,
                    gif_filename,
                    frontend,
                }
            }
        }
    }
}

impl RenderOutput for nes_headless_frame::Frame {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_universal_background(x, y, colour_index);
    }
}

struct NesRenderOutput<'a> {
    glutin_pixels: glutin_frontend::Pixels<'a>,
    gif_frame: &'a mut gif_renderer::Frame,
}

impl<'a> RenderOutput for glutin_frontend::Pixels<'a> {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour_universal_background(x, y, colour_index);
    }
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

fn render_to_hash<M: Mapper>(nes: &Nes<M>) -> u64 {
    let mut frame = nes_headless_frame::Frame::new();
    nes.render(&mut frame);
    let mut hasher = DefaultHasher::new();
    frame.hash(&mut hasher);
    hasher.finish()
}

#[derive(Serialize, Deserialize)]
pub enum DynamicNes {
    NromHorizontal(Nes<nrom::Nrom<nrom::Horizontal>>),
    NromVertical(Nes<nrom::Nrom<nrom::Vertical>>),
    Mmc1(Nes<mmc1::Mmc1>),
}

#[derive(Debug)]
pub enum Error {
    UnexpectedFormat(mapper::Error),
    InesParseError(ines::Error),
    DeserializeError(bincode::Error),
}

impl From<mapper::Error> for Error {
    fn from(e: mapper::Error) -> Self {
        Error::UnexpectedFormat(e)
    }
}

impl DynamicNes {
    fn from_ines(ines: &Ines) -> Result<Self, Error> {
        let &Ines {
            ref header,
            ref prg_rom,
            ref chr_rom,
        } = ines;
        use ines::Mapper::*;
        use ines::Mirroring::*;
        use mmc1::Mmc1;
        use nrom::Nrom;
        use DynamicNes as D;
        let mapper = header.mapper;
        let mirroring = header.mirroring;
        let dynamic_nes = match mapper {
            Nrom => match mirroring {
                Horizontal => {
                    D::NromHorizontal(Nes::new(Nrom::new(nrom::Horizontal, &prg_rom, &chr_rom)?))
                }
                Vertical => {
                    D::NromVertical(Nes::new(Nrom::new(nrom::Vertical, &prg_rom, &chr_rom)?))
                }
            },
            Mmc1 => D::Mmc1(Nes::new(Mmc1::new(&prg_rom, &chr_rom)?)),
        };
        Ok(dynamic_nes)
    }
    fn from_args(args: &Args) -> Result<Self, Error> {
        let rom_buffer = match &args.input {
            Input::Stdin => {
                let mut buffer = Vec::new();
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                handle
                    .read_to_end(&mut buffer)
                    .expect("Failed to read rom from stdin");
                buffer
            }
            Input::RomFile(rom_filename) => {
                let mut buffer = Vec::new();
                let mut rom_file = File::open(rom_filename).expect("Failed to open rom file");
                rom_file
                    .read_to_end(&mut buffer)
                    .expect("Failed to read rom file");
                buffer
            }
            Input::StateFile(state_filename) => {
                return load(&state_filename).map_err(Error::DeserializeError);
            }
        };
        Self::from_ines(&Ines::parse(&rom_buffer).map_err(Error::InesParseError)?)
    }
}

struct SaveConfig {
    filename: PathBuf,
    autosave_after_frames: Option<u64>,
}

struct AutosaveConfig {
    filename: PathBuf,
    autosave_after_frames: u64,
}

impl SaveConfig {
    fn autosave_config(&self) -> Option<AutosaveConfig> {
        self.autosave_after_frames
            .map(|autosave_after_frames| AutosaveConfig {
                autosave_after_frames,
                filename: self.filename.clone(),
            })
    }
}

struct Config {
    save_config: Option<SaveConfig>,
    gif_filename: Option<PathBuf>,
    kill_after_frames: Option<u64>,
}

impl Config {
    fn from_args(args: &Args) -> Self {
        let save_config = args
            .save_state_filename
            .as_ref()
            .map(|save_state_filename| SaveConfig {
                filename: save_state_filename.into(),
                autosave_after_frames: args.autosave_after_frames,
            });
        let gif_filename = args
            .gif_filename
            .as_ref()
            .map(|gif_filename| gif_filename.into());
        Self {
            save_config,
            gif_filename,
            kill_after_frames: args.kill_after_frames,
        }
    }
    fn save_filename(&self) -> Option<&PathBuf> {
        self.save_config
            .as_ref()
            .map(|save_config| &save_config.filename)
    }
    fn autosave_config(&self) -> Option<AutosaveConfig> {
        self.save_config
            .as_ref()
            .and_then(|save_config| save_config.autosave_config())
    }
}

fn save<M: Mapper + serde::Serialize, P: AsRef<Path>>(nes: &Nes<M>, path: Option<P>) {
    if let Some(path) = path {
        let bytes =
            bincode::serialize(&nes.clone_dynamic_nes()).expect("Failed to serialize state");
        let mut file = File::create(path).expect("Failed to create state file");
        file.write_all(&bytes).expect("Failed to write state file");
        println!("Wrote state file");
    } else {
        println!("No save state file specified");
    }
}

fn load<P: AsRef<Path>>(path: P) -> Result<DynamicNes, bincode::Error> {
    let mut state_file = File::open(path).expect("Failed to open state file");
    let mut bytes = Vec::new();
    state_file
        .read_to_end(&mut bytes)
        .expect("Failed to read state file");
    bincode::deserialize(&bytes)
}

enum Stop {
    Quit,
    Load(DynamicNes),
}

fn handle_event<M: Mapper + serde::ser::Serialize, P: AsRef<Path> + Copy>(
    nes: &mut Nes<M>,
    save_state_path: Option<P>,
    event: glutin::Event,
    frame_count: u64,
) -> Option<Stop> {
    match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            glutin::WindowEvent::CloseRequested => {
                return Some(Stop::Quit);
            }
            glutin::WindowEvent::KeyboardInput { input, .. } => match input.state {
                glutin::ElementState::Pressed => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        match virtual_keycode {
                            glutin::VirtualKeyCode::Left => {
                                nes::controller1::press::left(nes);
                            }
                            glutin::VirtualKeyCode::Right => {
                                nes::controller1::press::right(nes);
                            }
                            glutin::VirtualKeyCode::Up => {
                                nes::controller1::press::up(nes);
                            }
                            glutin::VirtualKeyCode::Down => {
                                nes::controller1::press::down(nes);
                            }
                            glutin::VirtualKeyCode::Return => {
                                nes::controller1::press::start(nes);
                            }
                            glutin::VirtualKeyCode::RShift => {
                                nes::controller1::press::select(nes);
                            }
                            glutin::VirtualKeyCode::A => {
                                nes::controller1::press::a(nes);
                            }
                            glutin::VirtualKeyCode::B => {
                                nes::controller1::press::b(nes);
                            }
                            glutin::VirtualKeyCode::S => save(&nes, save_state_path),
                            glutin::VirtualKeyCode::L => {
                                if let Some(save_state_path) = save_state_path.as_ref() {
                                    if let Some(dynamic_nes) = load(save_state_path).ok() {
                                        return Some(Stop::Load(dynamic_nes));
                                    }
                                }
                            }
                            glutin::VirtualKeyCode::I => {
                                println!("Frame Count: {}", frame_count);
                                println!("Frame Hash: {}", render_to_hash(nes));
                            }
                            _ => (),
                        }
                    }
                }
                glutin::ElementState::Released => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        match virtual_keycode {
                            glutin::VirtualKeyCode::Left => {
                                nes::controller1::release::left(nes);
                            }
                            glutin::VirtualKeyCode::Right => {
                                nes::controller1::release::right(nes);
                            }
                            glutin::VirtualKeyCode::Up => {
                                nes::controller1::release::up(nes);
                            }
                            glutin::VirtualKeyCode::Down => {
                                nes::controller1::release::down(nes);
                            }
                            glutin::VirtualKeyCode::Return => {
                                nes::controller1::release::start(nes);
                            }
                            glutin::VirtualKeyCode::RShift => {
                                nes::controller1::release::select(nes);
                            }
                            glutin::VirtualKeyCode::A => {
                                nes::controller1::release::a(nes);
                            }
                            glutin::VirtualKeyCode::B => {
                                nes::controller1::release::b(nes);
                            }
                            _ => (),
                        }
                    }
                }
            },
            _ => (),
        },
        _ => (),
    }
    None
}

fn run_glutin<M: Mapper + serde::ser::Serialize>(
    mut nes: Nes<M>,
    config: &Config,
    frontend: &mut glutin_frontend::Frontend,
) -> Stop {
    let mut frame_count = 0;
    let mut gif_renderer = config
        .gif_filename
        .as_ref()
        .map(|gif_filename| gif_renderer::Renderer::new(File::create(gif_filename).unwrap()));
    let autosave_config = config.autosave_config();
    loop {
        if Some(frame_count) == config.kill_after_frames {
            return Stop::Quit;
        }
        if let Some(autosave_config) = autosave_config.as_ref() {
            if frame_count == autosave_config.autosave_after_frames {
                save(&nes, Some(&autosave_config.filename));
            }
        }
        let mut stop = None;
        frontend.poll_glutin_events(|event| {
            if stop.is_none() {
                stop = handle_event(&mut nes, config.save_filename(), event, frame_count);
            }
        });
        if let Some(stop) = stop {
            return stop;
        };
        frontend.with_pixels(|mut pixels| {
            if let Some(gif_renderer) = gif_renderer.as_mut() {
                let mut gif_frame = gif_renderer::Frame::new();
                let mut render_output = NesRenderOutput {
                    glutin_pixels: pixels,
                    gif_frame: &mut gif_frame,
                };
                nes.render(&mut render_output);
                gif_renderer.add(gif_frame);
            } else {
                nes.render(&mut pixels);
            }
        });
        nes.run_for_frame();
        frontend.render();
        frame_count += 1;
    }
}

fn run_headless_hashing_final_frame<M: Mapper>(mut nes: Nes<M>, num_frames: u64) -> u64 {
    for _ in 0..num_frames {
        nes.run_for_frame();
    }
    render_to_hash(&nes)
}

fn run<M: Mapper + serde::ser::Serialize>(
    nes: Nes<M>,
    config: &Config,
    frontend: &mut Frontend,
) -> Stop {
    match frontend {
        Frontend::Glutin(glutin_frontend) => run_glutin(nes, config, glutin_frontend),
        Frontend::HeadlessPrintingFinalFrameHash { num_frames } => {
            let final_frame_hash = run_headless_hashing_final_frame(nes, *num_frames);
            println!("{:?}", final_frame_hash);
            Stop::Quit
        }
    }
}

fn main() {
    let args = Args::arg().with_help_default().parse_env_default_or_exit();
    let config = Config::from_args(&args);
    let mut current_nes = DynamicNes::from_args(&args).unwrap();
    let Args { mut frontend, .. } = args;
    loop {
        let stop = match current_nes {
            DynamicNes::NromHorizontal(nes) => run(nes, &config, &mut frontend),
            DynamicNes::NromVertical(nes) => run(nes, &config, &mut frontend),
            DynamicNes::Mmc1(nes) => run(nes, &config, &mut frontend),
        };
        match stop {
            Stop::Quit => break,
            Stop::Load(dynamic_nes) => current_nes = dynamic_nes,
        }
    }
}
