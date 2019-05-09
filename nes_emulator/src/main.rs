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

const RAM_BYTES: usize = 0x800;
const PALETTE_RAM_BYTES: usize = 0x20;
const NAME_TABLE_RAM_BYTES: usize = NAME_TABLE_BYTES * 2;

#[derive(Serialize, Deserialize)]
struct NesDevices {
    ram: Vec<u8>,
    rom: Vec<u8>,
    ppu: Ppu,
    ppu_memory: NesPpuMemory,
    controller1: Controller,
}

#[derive(Serialize, Deserialize)]
struct NesDevicesWithOam {
    devices: NesDevices,
    oam: Oam,
}

#[derive(Serialize, Deserialize)]
struct NesPpuMemory {
    name_table_ram: Vec<u8>,
    chr_rom: Vec<u8>,
    palette_ram: Vec<u8>,
}

impl PpuMemory for NesPpuMemory {
    fn write_u8(&mut self, address: PpuAddress, data: u8) {
        let address = address % 0x4000;
        match address {
            0x0000..=0x0FFF => println!("unimplemented pattern table write"),
            0x1000..=0x1FFF => println!("unimplemented pattern table write"),
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
        let address = address % 0x4000;
        match address {
            0x0000..=0x0FFF => self.chr_rom[address as usize],
            0x1000..=0x1FFF => self.chr_rom[address as usize],
            0x2000..=0x23FF => self.name_table_ram[address as usize - 0x2000],
            0x2400..=0x27FF => self.name_table_ram[address as usize - 0x2400],
            0x2800..=0x2BFF => self.name_table_ram[address as usize - 0x2400],
            0x2C00..=0x2FFF => self.name_table_ram[address as usize - 0x2800],
            0x3000..=0x33FF => self.name_table_ram[address as usize - 0x3000],
            0x3400..=0x37FF => self.name_table_ram[address as usize - 0x3400],
            0x3800..=0x3BFF => self.name_table_ram[address as usize - 0x3400],
            0x3C00..=0x3EFF => self.name_table_ram[address as usize - 0x3800],
            0x3F00..=0x3F1F => self.palette_ram[address as usize - 0x3F00],
            0x3F20..=0x3FFF => self.palette_ram[(address as usize - 0x3F20) % 0x20],
            _ => unreachable!(),
        }
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

#[derive(Debug, Serialize, Deserialize)]
struct Controller {
    current_state: u8,
    shift_register: u8,
    strobe: bool,
}

mod controller {
    pub mod bit {
        pub const A: u8 = 0;
        pub const B: u8 = 1;
        pub const SELECT: u8 = 2;
        pub const START: u8 = 3;
        pub const UP: u8 = 4;
        pub const DOWN: u8 = 5;
        pub const LEFT: u8 = 6;
        pub const RIGHT: u8 = 7;
    }
    pub mod flag {
        use super::bit;
        pub const A: u8 = 1 << bit::A;
        pub const B: u8 = 1 << bit::B;
        pub const SELECT: u8 = 1 << bit::SELECT;
        pub const START: u8 = 1 << bit::START;
        pub const UP: u8 = 1 << bit::UP;
        pub const DOWN: u8 = 1 << bit::DOWN;
        pub const LEFT: u8 = 1 << bit::LEFT;
        pub const RIGHT: u8 = 1 << bit::RIGHT;
    }
}

impl Controller {
    fn new() -> Self {
        Self {
            current_state: 0,
            shift_register: 0,
            strobe: false,
        }
    }
    fn set_strobe(&mut self) {
        self.shift_register = self.current_state;
        self.strobe = true;
    }
    fn clear_strobe(&mut self) {
        self.strobe = false;
    }
    fn is_strobe(&self) -> bool {
        self.strobe
    }
    fn shift_read(&mut self) -> u8 {
        let masked = self.shift_register & 1;
        self.shift_register = self.shift_register.wrapping_shr(1);
        masked
    }
    fn set_a(&mut self) {
        self.current_state |= controller::flag::A;
    }
    fn set_b(&mut self) {
        self.current_state |= controller::flag::B;
    }
    fn set_select(&mut self) {
        self.current_state |= controller::flag::SELECT;
    }
    fn set_start(&mut self) {
        self.current_state |= controller::flag::START;
    }
    fn set_left(&mut self) {
        self.current_state |= controller::flag::LEFT;
    }
    fn set_right(&mut self) {
        self.current_state |= controller::flag::RIGHT;
    }
    fn set_up(&mut self) {
        self.current_state |= controller::flag::UP;
    }
    fn set_down(&mut self) {
        self.current_state |= controller::flag::DOWN;
    }
    fn clear_a(&mut self) {
        self.current_state &= !controller::flag::A;
    }
    fn clear_b(&mut self) {
        self.current_state &= !controller::flag::B;
    }
    fn clear_select(&mut self) {
        self.current_state &= !controller::flag::SELECT;
    }
    fn clear_start(&mut self) {
        self.current_state &= !controller::flag::START;
    }
    fn clear_left(&mut self) {
        self.current_state &= !controller::flag::LEFT;
    }
    fn clear_right(&mut self) {
        self.current_state &= !controller::flag::RIGHT;
    }
    fn clear_up(&mut self) {
        self.current_state &= !controller::flag::UP;
    }
    fn clear_down(&mut self) {
        self.current_state &= !controller::flag::DOWN;
    }
}

impl Memory for NesDevices {
    fn read_u8(&mut self, address: Address) -> u8 {
        let data = match address {
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
            0x4016 => self.controller1.shift_read(),
            0x4000..=0x7FFF => {
                //println!("unimplemented read from {:x}", address);
                0
            }
            0x8000..=0xFFFF => self.rom[address as usize - 0x8000],
        };
        data
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
            0x4016 => {
                if data & 1 != 0 {
                    self.controller1.set_strobe();
                } else {
                    self.controller1.clear_strobe();
                }
            }
            0x4000..=0x7FFF => (),
            0x8000..=0xFFFF => panic!("unimplemented write {:x} to {:x}", data, address),
        }
    }
}

impl NesDevicesWithOam {
    fn print_oam(&self) {
        for i in 0..64 {
            let base = 0x200;
            let x = self.devices.ram[base + i * 4 + 3];
            let y = self.devices.ram[base + i * 4 + 0];
            let attributes = self.devices.ram[base + i * 4 + 2];
            let index = self.devices.ram[base + i * 4 + 1];
            println!(
                "{:02X}: {:02X} @ ({:03}, {:03}) (attr: {:02X} ",
                i, index, x, y, attributes
            );
        }
    }
}

impl Memory for NesDevicesWithOam {
    fn read_u8(&mut self, address: Address) -> u8 {
        self.devices.read_u8(address)
    }
    fn write_u8(&mut self, address: Address, data: u8) {
        if address == 0x4014 {
            self.oam.dma(&mut self.devices, data);
        } else {
            self.devices.write_u8(address, data);
        }
    }
}

impl MemoryReadOnly for NesDevices {
    fn read_u8_read_only(&self, address: Address) -> u8 {
        match address {
            0..=0x7FF => self.ram[address as usize],
            0x800..=0x7FFF => 0,
            0x8000..=0xFFFF => self.rom[(address as usize - 0x8000) % 0x4000],
        }
    }
}

impl MemoryReadOnly for NesDevicesWithOam {
    fn read_u8_read_only(&self, address: Address) -> u8 {
        self.devices.read_u8_read_only(address)
    }
}

#[derive(Serialize, Deserialize)]
struct Nes {
    cpu: Cpu,
    devices: NesDevicesWithOam,
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
    fn run_for_cycles_debug(&mut self, num_cycles: usize) {
        let mut count = 0;
        while count < num_cycles {
            let instruction_with_operand =
                InstructionWithOperand::next(&self.cpu, &self.devices).unwrap();
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let _ = writeln!(handle, "{}", instruction_with_operand);
            //writeln!(handle, "{:X?}", self.cpu).unwrap();
            count += self.cpu.step(&mut self.devices).unwrap() as usize;
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
        print_bytes_hex(&self.devices.devices.rom, 0x8000, 16);
        let _ = writeln!(handle, "\nRAM");
        print_bytes_hex(&self.devices.devices.ram, 0, 16);
        let _ = writeln!(handle, "\nVRAM");
        print_bytes_hex(&self.devices.devices.ppu_memory.name_table_ram, 0, 32);
        print_vram(&self.devices.devices.ppu_memory.name_table_ram);
        let _ = writeln!(handle, "PPU");
        let _ = writeln!(handle, "{:X?}", self.devices.devices.ppu);
    }
    fn analyse(&self) -> analyser::Analysis {
        let start = self
            .devices
            .read_u16_le_read_only(mos6502::interrupt_vector::START_LO);
        let nmi = self
            .devices
            .read_u16_le_read_only(mos6502::interrupt_vector::NMI_LO);
        let irq = self
            .devices
            .read_u16_le_read_only(mos6502::interrupt_vector::IRQ_LO);
        let indirect_jump_target_frame_start = 0xD4CC;
        analyser::Analysis::analyse(
            &self.devices,
            &NesMemoryMap,
            vec![start, nmi, irq, indirect_jump_target_frame_start],
        )
    }
}

struct NesMemoryMap;
impl analyser::MemoryMap for NesMemoryMap {
    fn normalize_function_call<M: MemoryReadOnly>(
        &self,
        jsr_opcode_address: Address,
        memory: &M,
    ) -> Option<Address> {
        if jsr_opcode_address >= 0x8000 {
            let function_definition_address =
                memory.read_u16_le_read_only(jsr_opcode_address.wrapping_add(1));
            if function_definition_address >= 0x8000 {
                if function_definition_address < 0xC000 {
                    Some(function_definition_address + 0x4000)
                } else {
                    Some(function_definition_address)
                }
            } else {
                None
            }
        } else {
            None
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

fn main() {
    let args = Args::arg().with_help_default().parse_env_default_or_exit();
    let mut frontend = Frontend::new();
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
    let Ines {
        prg_rom, chr_rom, ..
    } = Ines::parse(&buffer);
    let prg_rom = match prg_rom.len() {
        0x8000 => prg_rom,
        0x4000 => prg_rom.iter().chain(prg_rom.iter()).cloned().collect(),
        other => panic!("unexpected prg rom length {}", other),
    };
    let mut nes = if let Some(ref state_filename) = args.state_filename {
        let mut state_file = File::open(state_filename).expect("Failed to open state file");
        let mut bytes = Vec::new();
        state_file
            .read_to_end(&mut bytes)
            .expect("Failed to read state file");
        bincode::deserialize(&bytes).expect("Failed to parse state file")
    } else {
        let mut nes = Nes {
            cpu: Cpu::new(),
            devices: NesDevicesWithOam {
                devices: NesDevices {
                    ram: [0; RAM_BYTES].to_vec(),
                    rom: prg_rom.clone(),
                    ppu: Ppu::new(),
                    ppu_memory: NesPpuMemory {
                        name_table_ram: [0; NAME_TABLE_RAM_BYTES].to_vec(),
                        chr_rom: chr_rom.clone(),
                        palette_ram: [0; PALETTE_RAM_BYTES].to_vec(),
                    },
                    controller1: Controller::new(),
                },
                oam: Oam::new(),
            },
        };
        nes.start();
        nes
    };
    let mut running = true;
    let mut frame_count = 0;
    let mut output_gif_file = File::create("/tmp/a.gif").unwrap();
    let mut gif_renderer = gif_renderer::Renderer::new(output_gif_file);
    //nes.print_state();
    loop {
        if let Some(save_state_args) = args.save_state_args.as_ref() {
            if frame_count == save_state_args.frame {
                let bytes = bincode::serialize(&nes).expect("Failed to serialize state");
                let mut file =
                    File::create(&save_state_args.filename).expect("Failed to create state file");
                file.write_all(&bytes).expect("Failed to write state file");
                println!("Wrote state file");
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
                                        nes.devices.devices.controller1.set_left()
                                    }
                                    glutin::VirtualKeyCode::Right => {
                                        nes.devices.devices.controller1.set_right()
                                    }
                                    glutin::VirtualKeyCode::Up => {
                                        nes.devices.devices.controller1.set_up()
                                    }
                                    glutin::VirtualKeyCode::Down => {
                                        nes.devices.devices.controller1.set_down()
                                    }
                                    glutin::VirtualKeyCode::Return => {
                                        nes.devices.devices.controller1.set_start()
                                    }
                                    glutin::VirtualKeyCode::RShift => {
                                        nes.devices.devices.controller1.set_select()
                                    }
                                    glutin::VirtualKeyCode::A => {
                                        nes.devices.devices.controller1.set_a()
                                    }
                                    glutin::VirtualKeyCode::B => {
                                        nes.devices.devices.controller1.set_b()
                                    }
                                    glutin::VirtualKeyCode::S => {
                                        if let Some(save_state_args) = args.save_state_args.as_ref()
                                        {
                                            let bytes = bincode::serialize(&nes)
                                                .expect("Failed to serialize state");
                                            let mut file = File::create(&save_state_args.filename)
                                                .expect("Failed to create state file");
                                            file.write_all(&bytes)
                                                .expect("Failed to write state file");
                                            println!("Wrote state file");
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
                                        nes.devices.devices.controller1.clear_left()
                                    }
                                    glutin::VirtualKeyCode::Right => {
                                        nes.devices.devices.controller1.clear_right()
                                    }
                                    glutin::VirtualKeyCode::Up => {
                                        nes.devices.devices.controller1.clear_up()
                                    }
                                    glutin::VirtualKeyCode::Down => {
                                        nes.devices.devices.controller1.clear_down()
                                    }
                                    glutin::VirtualKeyCode::Return => {
                                        nes.devices.devices.controller1.clear_start()
                                    }
                                    glutin::VirtualKeyCode::RShift => {
                                        nes.devices.devices.controller1.clear_select()
                                    }
                                    glutin::VirtualKeyCode::A => {
                                        nes.devices.devices.controller1.clear_a()
                                    }
                                    glutin::VirtualKeyCode::B => {
                                        nes.devices.devices.controller1.clear_b()
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
            nes.devices.devices.ppu.render(
                &nes.devices.devices.ppu_memory,
                &nes.devices.oam,
                &mut render_output,
            );
            gif_renderer.add(gif_frame);
        });
        nes.run_for_cycles(30000);
        nes.nmi();
        frontend.render();
        frame_count += 1;
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
