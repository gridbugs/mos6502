use gif_renderer::Rgb24;
use ines::Ines;
use nes_emulator_core::{
    mapper::{Mapper, PersistentState},
    nes::{self, Nes},
    ppu::RenderOutput,
    DynamicNes, Error,
};
use nes_name_table_debug::NameTableFrame;
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
    fn parser() -> impl meap::Parser<Item = Self> {
        use meap::prelude::*;
        opt_opt::<u64, _>("INT", 'e')
            .name("nes-headless-frame")
            .desc("run with headless frontend, exiting after a specified number of frames")
            .map(|maybe_num_frames| {
                maybe_num_frames
                    .map(|num_frames| Self::HeadlessPrintingFinalFrameHash { num_frames })
                    .unwrap_or(Self::Graphical)
            })
    }
}

#[derive(Clone)]
enum Input {
    Stdin,
    RomFile(String),
    StateFile(String),
}

impl Input {
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::choose_at_most_one!(
            opt_opt::<String, _>("PATH", 'r')
                .name("rom-file")
                .desc("rom file (ines format) to load")
                .map(|path| path.map(Self::RomFile)),
            opt_opt::<String, _>("PATH", 'l')
                .name("load-state-file")
                .desc("state file to load")
                .map(|path| path.map(Self::StateFile)),
        )
        .with_default_general(Self::Stdin)
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
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                input = Input::parser();
                autosave_after_frames = opt_opt::<u64, _>("INT", 'a').name("autosave-after-frames").desc("save state after this many frames");
                kill_after_frames = opt_opt::<u64, _>("INT", 'k').name("kill-after-frames").desc("exit after this many frames");
                frame_duration = opt_opt::<u64, _>("INT", 'f').name("frame-duration-ms").desc("frame duration in milliseconds")
                    .map(|maybe_ms| maybe_ms.map(Duration::from_millis));
                save_state_filename = opt_opt::<String, _>("PATH", 's').name("save-state-file").desc("state file to save");
                gif_filename = opt_opt::<String, _>("PATH", 'g').name("gif").desc("gif file to record");
                name_table_gif_filename = opt_opt::<String, _>("PATH", 'n').name("name-table-gif").desc("gif file to record name tables into");
                frontend = Frontend::parser();
                debug = flag('d').name("debug").desc("enable debugging printouts");
                persistent_state_filename = opt_opt::<String, _>("PATH", 'p').name("persistent-state-filename").desc("file to store persistent state");
                zoom = opt_opt::<f64, _>("FLOAT", 'z').name("zoom").desc("real pixels per pixel").with_default(1.);
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

#[derive(Hash)]
struct NesHeadlessOutput(nes_headless_frame::Frame);
struct NesGraphicalOutput<'a>(graphical_frontend::Pixels<'a>);
struct NesGifOutput(gif_renderer::Frame);

impl RenderOutput for NesHeadlessOutput {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0
            .set_pixel_colour_universal_background(x, y, colour_index);
    }
}

impl<'a> RenderOutput for NesGraphicalOutput<'a> {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0
            .set_pixel_colour_universal_background(x, y, colour_index);
    }
}

impl RenderOutput for NesGifOutput {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0.set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.0
            .set_pixel_colour_universal_background(x, y, colour_index);
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

fn dynamic_nes_from_args(args: &Args) -> Result<DynamicNes, Error> {
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
    DynamicNes::from_ines(&Ines::parse(&rom_buffer).map_err(Error::InesParseError)?)
}

#[derive(Clone)]
struct SaveConfig {
    filename: PathBuf,
    autosave_after_frames: Option<u64>,
}

#[derive(Clone)]
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

#[derive(Clone)]
struct Config {
    save_config: Option<SaveConfig>,
    gif_filename: Option<PathBuf>,
    kill_after_frames: Option<u64>,
    frame_duration: Option<Duration>,
    debug: bool,
    persistent_state_filename: Option<PathBuf>,
    zoom: f64,
    name_table_gif_renderer: Option<String>,
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
        let name_table_gif_renderer = args.name_table_gif_filename.clone();
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
    event: graphical_frontend::input::Event,
) -> Option<MetaAction> {
    use graphical_frontend::input;
    match event {
        input::Event::WindowEvent { event, .. } => match event {
            input::WindowEvent::CloseRequested => {
                return Some(MetaAction::Stop(Stop::Quit));
            }
            input::WindowEvent::KeyboardInput { input, .. } => match input.state {
                input::ElementState::Pressed => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        match virtual_keycode {
                            input::VirtualKeyCode::Left => {
                                nes::controller1::press::left(nes);
                            }
                            input::VirtualKeyCode::Right => {
                                nes::controller1::press::right(nes);
                            }
                            input::VirtualKeyCode::Up => {
                                nes::controller1::press::up(nes);
                            }
                            input::VirtualKeyCode::Down => {
                                nes::controller1::press::down(nes);
                            }
                            input::VirtualKeyCode::Return => {
                                nes::controller1::press::start(nes);
                            }
                            input::VirtualKeyCode::RShift => {
                                nes::controller1::press::select(nes);
                            }
                            input::VirtualKeyCode::A => {
                                nes::controller1::press::a(nes);
                            }
                            input::VirtualKeyCode::B => {
                                nes::controller1::press::b(nes);
                            }
                            input::VirtualKeyCode::S => save(&nes, save_state_path),
                            input::VirtualKeyCode::L => {
                                if let Some(save_state_path) = save_state_path.as_ref() {
                                    if let Some(dynamic_nes) = load(save_state_path).ok() {
                                        return Some(MetaAction::Stop(Stop::Load(dynamic_nes)));
                                    }
                                }
                            }
                            input::VirtualKeyCode::P => {
                                if let Some(persistent_state_path) = persistent_state_path {
                                    if let Some(persistent_state) = nes.save_persistent_state() {
                                        save_persistent_state(
                                            &persistent_state,
                                            persistent_state_path,
                                        );
                                    }
                                }
                            }
                            input::VirtualKeyCode::I => {
                                return Some(MetaAction::PrintInfo);
                            }
                            _ => (),
                        }
                    }
                }
                input::ElementState::Released => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        match virtual_keycode {
                            input::VirtualKeyCode::Left => {
                                nes::controller1::release::left(nes);
                            }
                            input::VirtualKeyCode::Right => {
                                nes::controller1::release::right(nes);
                            }
                            input::VirtualKeyCode::Up => {
                                nes::controller1::release::up(nes);
                            }
                            input::VirtualKeyCode::Down => {
                                nes::controller1::release::down(nes);
                            }
                            input::VirtualKeyCode::Return => {
                                nes::controller1::release::start(nes);
                            }
                            input::VirtualKeyCode::RShift => {
                                nes::controller1::release::select(nes);
                            }
                            input::VirtualKeyCode::A => {
                                nes::controller1::release::a(nes);
                            }
                            input::VirtualKeyCode::B => {
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
    mut name_table_gif_renderer: Option<&mut NameTableGifRenderer>,
) {
    let name_table_frame = name_table_gif_renderer
        .as_mut()
        .map(|r| r.frame.deref_mut());
    if let Some(gif_renderer) = gif_renderer {
        let mut gif_frame = NesGifOutput(gif_renderer::Frame::new());
        let mut render_output = RenderOutputPair::new(pixels, &mut gif_frame);
        if config.debug {
            nes.run_for_frame_debug(&mut render_output, name_table_frame);
        } else {
            nes.run_for_frame(&mut render_output, name_table_frame);
        }
        #[cfg(feature = "ppu_debug")]
        {
            for ((x, y), age) in nes.ppu().debug().pixel_ages() {
                gif_frame.set_background_pixel_age(x, y, age);
            }
        }
        gif_renderer.add(&gif_frame.0);
    } else {
        if config.debug {
            nes.run_for_frame_debug(pixels, name_table_frame);
        } else {
            nes.run_for_frame(pixels, name_table_frame);
        }
    }
    if let Some(name_table_gif_renderer) = name_table_gif_renderer {
        name_table_gif_renderer.render();
    }
}

struct RunGraphical {
    dynamic_nes: DynamicNes,
    meta: RunGraphicalMeta,
}

struct RunGraphicalMeta {
    frame_count: u64,
    config: Config,
    gif_renderer: Option<gif_renderer::Renderer<File>>,
    name_table_gif_renderer: Option<NameTableGifRenderer>,
    print_info: bool,
}

impl RunGraphicalMeta {
    fn tick_gen<M: Mapper + serde::ser::Serialize>(
        &mut self,
        nes: &mut Nes<M>,
        pixels: graphical_frontend::Pixels,
    ) -> Option<graphical_frontend::ControlFlow> {
        let realtime_frame_timing = self
            .config
            .frame_duration
            .map(|frame_duration| (frame_duration, Instant::now()));
        if Some(self.frame_count) == self.config.kill_after_frames {
            return Some(graphical_frontend::ControlFlow::Quit);
        }
        if let Some(autosave_config) = self.config.autosave_config() {
            if self.frame_count == autosave_config.autosave_after_frames {
                save(&nes, Some(&autosave_config.filename));
            }
        }
        let mut pixels = NesGraphicalOutput(pixels);
        if self.print_info {
            let mut memory_only_frame = NesHeadlessOutput(nes_headless_frame::Frame::new());
            let mut render_output = RenderOutputPair::new(&mut pixels, &mut memory_only_frame);
            run_nes_for_frame(
                nes,
                &mut self.config,
                &mut render_output,
                self.gif_renderer.as_mut(),
                self.name_table_gif_renderer.as_mut(),
            );
            let mut hasher = DefaultHasher::new();
            memory_only_frame.hash(&mut hasher);
            let frame_hash = hasher.finish();
            println!("Frame Count: {}", self.frame_count);
            println!("Frame Hash: {}", frame_hash);
            self.print_info = false;
        } else {
            run_nes_for_frame(
                nes,
                &mut self.config,
                &mut pixels,
                self.gif_renderer.as_mut(),
                self.name_table_gif_renderer.as_mut(),
            );
        }
        if let Some((frame_duration, frame_start)) = realtime_frame_timing {
            if let Some(remaining) = frame_duration.checked_sub(frame_start.elapsed()) {
                thread::sleep(remaining);
            }
        }
        self.frame_count += 1;
        None
    }
}

impl graphical_frontend::AppTrait for RunGraphical {
    fn handle_input(
        &mut self,
        e: graphical_frontend::input::Event,
    ) -> Option<graphical_frontend::ControlFlow> {
        let s = self.meta.config.save_filename();
        let p = self.meta.config.persistent_state_filename.as_ref();
        let meta_action = match self.dynamic_nes {
            DynamicNes::NromHorizontal(ref mut n) => handle_event(n, s, p, e),
            DynamicNes::NromVertical(ref mut n) => handle_event(n, s, p, e),
            DynamicNes::Mmc1(ref mut n) => handle_event(n, s, p, e),
        };
        match meta_action {
            None => None,
            Some(MetaAction::PrintInfo) => {
                self.meta.print_info = true;
                None
            }
            Some(MetaAction::Stop(stop)) => match stop {
                Stop::Quit => Some(graphical_frontend::ControlFlow::Quit),
                Stop::Load(dynamic_nes) => {
                    self.dynamic_nes = dynamic_nes;
                    None
                }
            },
        }
    }
    fn tick(&mut self, p: graphical_frontend::Pixels) -> Option<graphical_frontend::ControlFlow> {
        let m = &mut self.meta;
        match self.dynamic_nes {
            DynamicNes::NromHorizontal(ref mut n) => m.tick_gen(n, p),
            DynamicNes::NromVertical(ref mut n) => m.tick_gen(n, p),
            DynamicNes::Mmc1(ref mut n) => m.tick_gen(n, p),
        }
    }
}

fn run_headless_hashing_final_frame_gen<M: Mapper>(mut nes: Nes<M>, num_frames: u64) -> u64 {
    if let Some(n) = num_frames.checked_sub(1) {
        for _ in 0..n {
            nes.run_for_frame(&mut NoRenderOutput, None);
        }
    }
    let mut frame = NesHeadlessOutput(nes_headless_frame::Frame::new());
    nes.run_for_frame(&mut frame, None);
    let mut hasher = DefaultHasher::new();
    frame.hash(&mut hasher);
    hasher.finish()
}

fn run_headless_hashing_final_frame(dynamic_nes: DynamicNes, num_frames: u64) -> u64 {
    match dynamic_nes {
        DynamicNes::NromHorizontal(n) => run_headless_hashing_final_frame_gen(n, num_frames),
        DynamicNes::NromVertical(n) => run_headless_hashing_final_frame_gen(n, num_frames),
        DynamicNes::Mmc1(n) => run_headless_hashing_final_frame_gen(n, num_frames),
    }
}

fn main() {
    use meap::Parser;
    env_logger::init();
    let args = Args::parser().with_help_default().parse_env_or_exit();
    let config = Config::from_args(&args);
    let mut dynamic_nes = dynamic_nes_from_args(&args).unwrap();
    let Args { frontend, .. } = args;
    match frontend {
        Frontend::HeadlessPrintingFinalFrameHash { num_frames } => {
            let final_frame_hash = run_headless_hashing_final_frame(dynamic_nes, num_frames);
            println!("{}", final_frame_hash);
        }
        Frontend::Graphical => {
            let graphical_frontend = graphical_frontend::Frontend::new(config.zoom);
            let gif_renderer = config.gif_filename.as_ref().map(|gif_filename| {
                gif_renderer::Renderer::new(File::create(gif_filename).unwrap())
            });
            let name_table_gif_renderer = config
                .name_table_gif_renderer
                .as_ref()
                .map(|f| NameTableGifRenderer::new(f));
            if let Some(persistent_state_filename) = config.persistent_state_filename.as_ref() {
                if let Some(Ok(persistent_state)) = load_persistent_state(persistent_state_filename)
                {
                    dynamic_nes
                        .load_persistent_state(&persistent_state)
                        .unwrap();
                }
            }
            let run_graphical = RunGraphical {
                dynamic_nes,
                meta: RunGraphicalMeta {
                    frame_count: 0,
                    config,
                    gif_renderer,
                    name_table_gif_renderer,
                    print_info: false,
                },
            };
            graphical_frontend.run(run_graphical);
        }
    }
}
