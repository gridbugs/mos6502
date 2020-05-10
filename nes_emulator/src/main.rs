mod apu;
mod mapper;
mod nes;
mod ppu;
mod timing;

use gif_renderer::Rgb24;
use graphical_frontend::glutin;
use ines::Ines;
use mapper::{mmc1, nrom, Mapper, PersistentState};
use nes::Nes;
use nes_name_table_debug::NameTableFrame;
use ppu::RenderOutput;
use serde::{Deserialize, Serialize};
use simon::{args_map, Arg};
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
enum Frontend {
    Graphical,
    HeadlessPrintingFinalFrameHash { num_frames: u64 },
}

impl Frontend {
    fn arg() -> impl Arg<Item = Self> {
        simon::opt(
            "e",
            "headless-num-frames",
            "run with headless frontend, exiting after a specified number of frames",
            "INT",
        )
        .option_map(|num_frames| Frontend::HeadlessPrintingFinalFrameHash { num_frames })
        .with_default(Frontend::Graphical)
    }
}

#[derive(Clone)]
enum Input {
    Stdin,
    RomFile(String),
    StateFile(String),
}

impl Input {
    fn arg() -> impl Arg<Item = Self> {
        let rom_file = simon::opt("r", "rom-file", "rom file (ines format) to load", "PATH")
            .option_map(Input::RomFile);
        let load_state_file = simon::opt("l", "load-state-file", "state file to load", "PATH")
            .option_map(Input::StateFile);
        rom_file.choice(load_state_file).with_default(Input::Stdin)
    }
}

struct Args {
    input: Input,
    autosave_after_frames: Option<u64>,
    kill_after_frames: Option<u64>,
    frame_duration: Option<Duration>,
    save_state_filename: Option<String>,
    gif_filename: Option<String>,
    name_table_gif_filename: Option<String>,
    frontend: Frontend,
    debug: bool,
    persistent_state_filename: Option<String>,
    zoom: f64,
}

impl Args {
    fn arg() -> impl Arg<Item = Self> {
        args_map! {
            let {
                input = Input::arg();
                autosave_after_frames = simon::opt("a", "autosave-after-frames", "save state after this many frames", "INT");
                kill_after_frames = simon::opt("k", "kill-after-frames", "exit after this many frames", "INT");
                frame_duration = simon::opt("f", "frame-duration-ms", "frame duration in milliseconds", "INT")
                    .option_map(Duration::from_millis);
                save_state_filename = simon::opt("s", "save-state-file", "state file to save", "PATH");
                gif_filename = simon::opt("g", "gif", "gif file to record into", "PATH");
                name_table_gif_filename = simon::opt("n", "name-table-gif", "gif file to record name tables into", "PATH");
                frontend = Frontend::arg();
                debug = simon::flag("d", "debug", "enable debugging printouts");
                persistent_state_filename = simon::opt("p", "persistent-state-filename", "file to store persistent state", "PATH");
                zoom = simon::opt("z", "zoom", "real pixels per nes pixel", "FLOAT").with_default(2.);
            } in {
                Self {
                    input,
                    autosave_after_frames,
                    kill_after_frames,
                    frame_duration,
                    save_state_filename,
                    gif_filename,
                    name_table_gif_filename,
                    frontend,
                    debug,
                    persistent_state_filename,
                    zoom,
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

impl<'a> RenderOutput for graphical_frontend::Pixels<'a> {
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

struct RenderOutputPair<'a, A, B> {
    a: &'a mut A,
    b: &'a mut B,
}

impl<'a, A, B> RenderOutputPair<'a, A, B> {
    fn new(a: &'a mut A, b: &'a mut B) -> Self {
        Self { a, b }
    }
}

impl<'a, A, B> RenderOutput for RenderOutputPair<'a, A, B>
where
    A: RenderOutput,
    B: RenderOutput,
{
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a.set_pixel_colour_sprite_back(x, y, colour_index);
        self.b.set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a.set_pixel_colour_sprite_front(x, y, colour_index);
        self.b.set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a.set_pixel_colour_background(x, y, colour_index);
        self.b.set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a
            .set_pixel_colour_universal_background(x, y, colour_index);
        self.b
            .set_pixel_colour_universal_background(x, y, colour_index);
    }
}

struct NoRenderOutput;

impl RenderOutput for NoRenderOutput {
    fn set_pixel_colour_sprite_back(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
    fn set_pixel_colour_sprite_front(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
    fn set_pixel_colour_background(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
    fn set_pixel_colour_universal_background(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
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
    frame_duration: Option<Duration>,
    debug: bool,
    persistent_state_filename: Option<PathBuf>,
    zoom: f64,
    name_table_gif_renderer: Option<NameTableGifRenderer>,
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
        let persistent_state_filename = args.persistent_state_filename.as_ref().map(|f| f.into());
        let name_table_gif_renderer = args
            .name_table_gif_filename
            .as_ref()
            .map(|f| NameTableGifRenderer::new(f));
        Self {
            save_config,
            gif_filename,
            kill_after_frames: args.kill_after_frames,
            frame_duration: args.frame_duration,
            debug: args.debug,
            persistent_state_filename,
            zoom: args.zoom,
            name_table_gif_renderer,
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

fn save_persistent_state<P: AsRef<Path>>(persistent_state: &PersistentState, path: P) {
    let bytes = bincode::serialize(persistent_state).expect("Failed to serialize persistent state");
    let mut file = File::create(path).expect("Failed to create persistent state file");
    file.write_all(&bytes)
        .expect("Failed to write persistent state file");
    println!("Wrote persistent state file");
}

fn load_persistent_state<P: AsRef<Path>>(
    path: P,
) -> Option<Result<PersistentState, bincode::Error>> {
    if path.as_ref().exists() {
        let mut persistent_state_file =
            File::open(path).expect("Failed to open persistent state file");
        let mut bytes = Vec::new();
        persistent_state_file
            .read_to_end(&mut bytes)
            .expect("Failed to read persistent state file");
        Some(bincode::deserialize(&bytes))
    } else {
        None
    }
}

enum Stop {
    Quit,
    Load(DynamicNes),
}

enum MetaAction {
    Stop(Stop),
    PrintInfo,
}

fn handle_event<M: Mapper + serde::ser::Serialize, P: AsRef<Path> + Copy, Q: AsRef<Path> + Copy>(
    nes: &mut Nes<M>,
    save_state_path: Option<P>,
    persistent_state_path: Option<Q>,
    event: glutin::Event,
) -> Option<MetaAction> {
    match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            glutin::WindowEvent::CloseRequested => {
                return Some(MetaAction::Stop(Stop::Quit));
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
                                        return Some(MetaAction::Stop(Stop::Load(dynamic_nes)));
                                    }
                                }
                            }
                            glutin::VirtualKeyCode::P => {
                                if let Some(persistent_state_path) = persistent_state_path {
                                    if let Some(persistent_state) = nes.save_persistent_state() {
                                        save_persistent_state(
                                            &persistent_state,
                                            persistent_state_path,
                                        );
                                    }
                                }
                            }
                            glutin::VirtualKeyCode::I => {
                                return Some(MetaAction::PrintInfo);
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

struct NameTableGifRenderer {
    gif_renderer: gif_renderer::NameTableRenderer<File>,
    frame: Box<NameTableFrame>,
}

impl NameTableGifRenderer {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        let gif_renderer = gif_renderer::NameTableRenderer::new(
            File::create(path).unwrap(),
            |on_screen| on_screen.saturating_add(Rgb24::new(50, 50, 100)),
            |off_screen| off_screen.saturating_scalar_mul_div(1, 2),
        );
        let frame = Box::new(NameTableFrame::new());
        Self {
            gif_renderer,
            frame,
        }
    }
    fn render(&mut self) {
        self.gif_renderer.add_name_table_frame(&self.frame);
        *self.frame = NameTableFrame::new();
    }
}

fn run_nes_for_frame<M: Mapper, O: RenderOutput>(
    nes: &mut Nes<M>,
    config: &mut Config,
    pixels: &mut O,
    gif_renderer: Option<&mut gif_renderer::Renderer<File>>,
) {
    let name_table_frame = config
        .name_table_gif_renderer
        .as_mut()
        .map(|r| r.frame.deref_mut());
    if let Some(gif_renderer) = gif_renderer {
        let mut gif_frame = gif_renderer::Frame::new();
        let mut render_output = RenderOutputPair::new(pixels, &mut gif_frame);
        if config.debug {
            nes.run_for_frame_debug(&mut render_output, name_table_frame);
        } else {
            nes.run_for_frame(&mut render_output, name_table_frame);
        }
        gif_renderer.add(&gif_frame);
    } else {
        if config.debug {
            nes.run_for_frame_debug(pixels, name_table_frame);
        } else {
            nes.run_for_frame(pixels, name_table_frame);
        }
    }
    if let Some(name_table_gif_renderer) = config.name_table_gif_renderer.as_mut() {
        name_table_gif_renderer.render();
    }
}

fn run_graphical<M: Mapper + serde::ser::Serialize>(
    mut nes: Nes<M>,
    config: &mut Config,
    frontend: &mut graphical_frontend::Frontend,
) -> Stop {
    let mut frame_count = 0;
    let mut gif_renderer = config
        .gif_filename
        .as_ref()
        .map(|gif_filename| gif_renderer::Renderer::new(File::create(gif_filename).unwrap()));
    let autosave_config = config.autosave_config();
    loop {
        let realtime_frame_timing = config
            .frame_duration
            .map(|frame_duration| (frame_duration, Instant::now()));
        if Some(frame_count) == config.kill_after_frames {
            return Stop::Quit;
        }
        if let Some(autosave_config) = autosave_config.as_ref() {
            if frame_count == autosave_config.autosave_after_frames {
                save(&nes, Some(&autosave_config.filename));
            }
        }
        let mut meta_action = None;
        frontend.poll_events(|event| {
            if meta_action.is_none() {
                meta_action = handle_event(
                    &mut nes,
                    config.save_filename(),
                    config.persistent_state_filename.as_ref(),
                    event,
                );
            }
        });
        if let Some(meta_action) = meta_action {
            match meta_action {
                MetaAction::Stop(stop) => return stop,
                MetaAction::PrintInfo => {
                    frontend.with_pixels(|mut pixels| {
                        let mut memory_only_frame = nes_headless_frame::Frame::new();
                        let mut render_output =
                            RenderOutputPair::new(&mut pixels, &mut memory_only_frame);
                        run_nes_for_frame(
                            &mut nes,
                            config,
                            &mut render_output,
                            gif_renderer.as_mut(),
                        );
                        let mut hasher = DefaultHasher::new();
                        memory_only_frame.hash(&mut hasher);
                        let frame_hash = hasher.finish();
                        println!("Frame Count: {}", frame_count);
                        println!("Frame Hash: {}", frame_hash);
                    });
                }
            }
        } else {
            frontend.with_pixels(|mut pixels| {
                run_nes_for_frame(&mut nes, config, &mut pixels, gif_renderer.as_mut());
            });
        }
        if let Some((frame_duration, frame_start)) = realtime_frame_timing {
            if let Some(remaining) = frame_duration.checked_sub(frame_start.elapsed()) {
                thread::sleep(remaining);
            }
        }
        frontend.render();
        frame_count += 1;
    }
}

fn run_headless_hashing_final_frame<M: Mapper>(mut nes: Nes<M>, num_frames: u64) -> u64 {
    if let Some(n) = num_frames.checked_sub(1) {
        for _ in 0..n {
            nes.run_for_frame(&mut NoRenderOutput, None);
        }
    }
    let mut frame = nes_headless_frame::Frame::new();
    nes.run_for_frame(&mut frame, None);
    let mut hasher = DefaultHasher::new();
    frame.hash(&mut hasher);
    hasher.finish()
}

#[derive(Default)]
struct LazyFrontendResources {
    graphical: Option<graphical_frontend::Frontend>,
}

fn run<M: Mapper + serde::ser::Serialize>(
    mut nes: Nes<M>,
    config: &mut Config,
    frontend: &Frontend,
    frontend_resources: &mut LazyFrontendResources,
) -> Stop {
    if let Some(persistent_state_filename) = config.persistent_state_filename.as_ref() {
        if let Some(Ok(persistent_state)) = load_persistent_state(persistent_state_filename) {
            nes.load_persistent_state(&persistent_state).unwrap();
        }
    }
    match frontend {
        Frontend::Graphical => {
            if frontend_resources.graphical.is_none() {
                frontend_resources.graphical = Some(graphical_frontend::Frontend::new(config.zoom));
            }
            run_graphical(nes, config, frontend_resources.graphical.as_mut().unwrap())
        }
        Frontend::HeadlessPrintingFinalFrameHash { num_frames } => {
            let final_frame_hash = run_headless_hashing_final_frame(nes, *num_frames);
            println!("{}", final_frame_hash);
            Stop::Quit
        }
    }
}

fn main() {
    let args = Args::arg().with_help_default().parse_env_or_exit();
    let mut config = Config::from_args(&args);
    let mut current_nes = DynamicNes::from_args(&args).unwrap();
    let mut res = LazyFrontendResources::default();
    let Args { frontend, .. } = args;
    loop {
        let stop = match current_nes {
            DynamicNes::NromHorizontal(nes) => run(nes, &mut config, &frontend, &mut res),
            DynamicNes::NromVertical(nes) => run(nes, &mut config, &frontend, &mut res),
            DynamicNes::Mmc1(nes) => run(nes, &mut config, &frontend, &mut res),
        };
        match stop {
            Stop::Quit => break,
            Stop::Load(dynamic_nes) => current_nes = dynamic_nes,
        }
    }
}
