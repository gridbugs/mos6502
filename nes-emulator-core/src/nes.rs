use crate::apu::Apu;
use crate::dynamic_nes::DynamicNes;
use crate::mapper::{Mapper, PersistentState, PersistentStateError};
use crate::ppu::{Oam, Ppu, ScanlineIter};
use crate::timing;
use mos6502_model::debug::InstructionWithOperand;
use mos6502_model::machine::{Address, Cpu, Memory, MemoryReadOnly};
use nes_name_table_debug::NameTableFrame;
use nes_render_output::RenderOutput;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::io::{self, Write};

const RAM_BYTES: usize = 0x800;

#[derive(Clone, Serialize, Deserialize)]
struct NesDevices<M: Mapper> {
    #[serde(with = "BigArray")]
    ram: [u8; RAM_BYTES],
    ppu: Ppu,
    apu: Apu,
    controller1: Controller,
    mapper: M,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NesDevicesWithOam<M: Mapper> {
    devices: NesDevices<M>,
    oam: Oam,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Controller {
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
    fn shift_read(&mut self) -> u8 {
        let masked = self.shift_register & 1;
        self.shift_register = self.shift_register.wrapping_shr(1);
        masked
    }
    pub fn set_a(&mut self) {
        self.current_state |= controller::flag::A;
    }
    pub fn set_b(&mut self) {
        self.current_state |= controller::flag::B;
    }
    pub fn set_select(&mut self) {
        self.current_state |= controller::flag::SELECT;
    }
    pub fn set_start(&mut self) {
        self.current_state |= controller::flag::START;
    }
    pub fn set_left(&mut self) {
        self.current_state |= controller::flag::LEFT;
    }
    pub fn set_right(&mut self) {
        self.current_state |= controller::flag::RIGHT;
    }
    pub fn set_up(&mut self) {
        self.current_state |= controller::flag::UP;
    }
    pub fn set_down(&mut self) {
        self.current_state |= controller::flag::DOWN;
    }
    pub fn clear_a(&mut self) {
        self.current_state &= !controller::flag::A;
    }
    pub fn clear_b(&mut self) {
        self.current_state &= !controller::flag::B;
    }
    pub fn clear_select(&mut self) {
        self.current_state &= !controller::flag::SELECT;
    }
    pub fn clear_start(&mut self) {
        self.current_state &= !controller::flag::START;
    }
    pub fn clear_left(&mut self) {
        self.current_state &= !controller::flag::LEFT;
    }
    pub fn clear_right(&mut self) {
        self.current_state &= !controller::flag::RIGHT;
    }
    pub fn clear_up(&mut self) {
        self.current_state &= !controller::flag::UP;
    }
    pub fn clear_down(&mut self) {
        self.current_state &= !controller::flag::DOWN;
    }
}

impl<M: Mapper> Memory for NesDevices<M> {
    fn read_u8(&mut self, address: Address) -> u8 {
        let data = match address {
            0..=0x1FFF => self.ram[address as usize % RAM_BYTES],
            0x2000..=0x3FFF => match address % 8 {
                0 => 0,
                1 => 0,
                2 => self.ppu.read_status(),
                3 => 0,
                5 => 0,
                6 => 0,
                7 => self.ppu.read_data(&self.mapper),
                _ => unreachable!(),
            },
            0x4016 => self.controller1.shift_read(),
            0x4000..=0x401F => 0,
            cartridge_address => self.mapper.cpu_read_u8(cartridge_address),
        };
        data
    }
    fn read_u8_zero_page(&mut self, address: u8) -> u8 {
        self.ram[address as usize]
    }
    fn read_u8_stack(&mut self, stack_pointer: u8) -> u8 {
        self.ram[0x0100 | stack_pointer as usize]
    }
    fn write_u8(&mut self, address: Address, data: u8) {
        match address {
            0..=0x1FFF => self.ram[address as usize % RAM_BYTES] = data,
            0x2000..=0x3FFF => match address % 8 {
                0 => self.ppu.write_control(data),
                1 => self.ppu.write_mask(data),
                2 => (),
                3 => self.ppu.write_oam_address(data),
                5 => self.ppu.write_scroll(data),
                6 => self.ppu.write_address(data),
                7 => self.ppu.write_data(&mut self.mapper, data),
                _ => unreachable!(),
            },
            0x4016 => {
                if data & 1 != 0 {
                    self.controller1.set_strobe();
                } else {
                    self.controller1.clear_strobe();
                }
            }
            0x4000..=0x401F => {}
            cartridge_address => self.mapper.cpu_write_u8(cartridge_address, data),
        }
    }
    fn write_u8_zero_page(&mut self, address: u8, data: u8) {
        self.ram[address as usize] = data;
    }
    fn write_u8_stack(&mut self, stack_pointer: u8, data: u8) {
        self.ram[0x0100 | stack_pointer as usize] = data;
    }
}

impl<M: Mapper> Memory for NesDevicesWithOam<M> {
    fn read_u8(&mut self, address: Address) -> u8 {
        match address {
            0x2004 => self.devices.ppu.read_oam_data(&self.oam),
            other => self.devices.read_u8(other),
        }
    }
    fn write_u8(&mut self, address: Address, data: u8) {
        match address {
            0x4014 => self.oam.dma(&mut self.devices, data),
            0x2004 => self.devices.ppu.write_oam_data(data, &mut self.oam),
            other => self.devices.write_u8(other, data),
        }
    }
    fn read_u8_zero_page(&mut self, address: u8) -> u8 {
        self.devices.read_u8_zero_page(address)
    }
    fn read_u8_stack(&mut self, stack_pointer: u8) -> u8 {
        self.devices.read_u8_stack(stack_pointer)
    }
    fn write_u8_zero_page(&mut self, address: u8, data: u8) {
        self.devices.write_u8_zero_page(address, data);
    }
    fn write_u8_stack(&mut self, stack_pointer: u8, data: u8) {
        self.devices.write_u8_stack(stack_pointer, data);
    }
}

impl<M: Mapper> MemoryReadOnly for NesDevices<M> {
    fn read_u8_read_only(&self, address: Address) -> u8 {
        let data = match address {
            0..=0x1FFF => self.ram[address as usize % RAM_BYTES],
            0x2000..=0x401F => 0,
            cartridge_address => self.mapper.cpu_read_u8_read_only(cartridge_address),
        };
        data
    }
}

impl<M: Mapper> MemoryReadOnly for NesDevicesWithOam<M> {
    fn read_u8_read_only(&self, address: Address) -> u8 {
        self.devices.read_u8_read_only(address)
    }
}

pub trait RunForCycles {
    fn run_for_cycles<M: Memory + MemoryReadOnly>(
        &mut self,
        cpu: &mut Cpu,
        memory: &mut M,
        num_cycles: u32,
    );
}

pub struct RunForCyclesRegular;

pub struct RunForCyclesDebug;

impl RunForCycles for RunForCyclesRegular {
    fn run_for_cycles<M: Memory + MemoryReadOnly>(
        &mut self,
        cpu: &mut Cpu,
        memory: &mut M,
        num_cycles: u32,
    ) {
        cpu.run_for_cycles(memory, num_cycles as usize).unwrap();
    }
}

impl RunForCycles for RunForCyclesDebug {
    fn run_for_cycles<M: Memory + MemoryReadOnly>(
        &mut self,
        cpu: &mut Cpu,
        memory: &mut M,
        num_cycles: u32,
    ) {
        let mut count = 0;
        while count < num_cycles {
            if let Ok(instruction_with_operand) = InstructionWithOperand::next(cpu, memory) {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                let _ = writeln!(handle, "{}", instruction_with_operand);
            }
            count += cpu.step(memory).unwrap() as u32;
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Nes<M: Mapper> {
    cpu: Cpu,
    devices: NesDevicesWithOam<M>,
}

impl<M: Mapper> Nes<M> {
    fn start(&mut self) {
        self.cpu.start(&mut self.devices);
    }
    pub fn new(mapper: M) -> Self {
        let mut nes = Nes {
            cpu: Cpu::new(),
            devices: NesDevicesWithOam {
                devices: NesDevices {
                    ram: [0; RAM_BYTES],
                    ppu: Ppu::new(),
                    apu: Apu::new(),
                    controller1: Controller::new(),
                    mapper,
                },
                oam: Oam::new(),
            },
        };
        nes.start();
        nes
    }
    pub fn run_for_frame_general<R: RunForCycles, O: RenderOutput>(
        &mut self,
        run: &mut R,
        pixels: &mut O,
        mut name_table_frame: Option<&mut NameTableFrame>,
    ) {
        // pre-render scanline
        run.run_for_cycles(
            &mut self.cpu,
            &mut self.devices,
            timing::ntsc::APPROX_CPU_CYCLES_PER_SCANLINE,
        );
        self.devices.devices.ppu.render_sprites(
            &self.devices.devices.mapper,
            &self.devices.oam,
            pixels,
        );
        if let Some(ref mut name_table_frame) = name_table_frame {
            self.devices
                .devices
                .ppu
                .debug_render_name_table_frame(&self.devices.devices.mapper, name_table_frame);
        }
        let sprite_zero = self
            .devices
            .devices
            .ppu
            .sprite_zero(&self.devices.oam, &mut self.devices.devices.mapper);
        for scanline in ScanlineIter::new() {
            if let Some(ref mut name_table_frame) = name_table_frame {
                name_table_frame.set_scroll(
                    scanline.index(),
                    self.devices.devices.ppu.scroll_x(),
                    self.devices.devices.ppu.scroll_y(),
                );
            }
            run.run_for_cycles(
                &mut self.cpu,
                &mut self.devices,
                timing::ntsc::APPROX_CPU_CYCLES_PER_SCANLINE,
            );
            if let Some(sprite_zero_hit) = self.devices.devices.ppu.render_background_scanline(
                scanline,
                &sprite_zero,
                &self.devices.devices.mapper,
                pixels,
            ) {
                let pixels_after_sprite_zero_hit =
                    nes_specs::SCREEN_WIDTH_PX - sprite_zero_hit.screen_pixel_x() as u16;
                let approx_cpu_cycles_after_sprite_zero_hit = pixels_after_sprite_zero_hit as u32
                    / timing::ntsc::NUM_PPU_CYCLES_PER_CPU_CYCLE;
                run.run_for_cycles(
                    &mut self.cpu,
                    &mut self.devices,
                    approx_cpu_cycles_after_sprite_zero_hit,
                );
            }
        }
        // post-render scanline
        run.run_for_cycles(
            &mut self.cpu,
            &mut self.devices,
            timing::ntsc::APPROX_CPU_CYCLES_PER_SCANLINE,
        );
        if self.devices.devices.ppu.is_vblank_nmi_enabled() {
            self.cpu.nmi(&mut self.devices);
        }
        self.devices.devices.ppu.before_vblank();
        run.run_for_cycles(
            &mut self.cpu,
            &mut self.devices,
            timing::ntsc::APPROX_CPU_CYCLES_PER_VBLANK,
        );
        self.devices.devices.ppu.after_vblank();
    }
    pub fn run_for_frame<O: RenderOutput>(
        &mut self,
        pixels: &mut O,
        name_table_frame: Option<&mut NameTableFrame>,
    ) {
        self.run_for_frame_general(&mut RunForCyclesRegular, pixels, name_table_frame);
    }
    pub fn run_for_frame_debug<O: RenderOutput>(
        &mut self,
        pixels: &mut O,
        name_table_frame: Option<&mut NameTableFrame>,
    ) {
        self.run_for_frame_general(&mut RunForCyclesDebug, pixels, name_table_frame);
    }
    pub fn clone_dynamic_nes(&self) -> DynamicNes {
        M::clone_dynamic_nes(self)
    }
    pub fn save_persistent_state(&self) -> Option<PersistentState> {
        self.devices.devices.mapper.save_persistent_state()
    }
    pub fn load_persistent_state(
        &mut self,
        persistent_state: &PersistentState,
    ) -> Result<(), PersistentStateError> {
        self.devices
            .devices
            .mapper
            .load_persistent_state(persistent_state)
    }
    pub fn ppu(&self) -> &Ppu {
        &self.devices.devices.ppu
    }
    pub fn controller1_mut(&mut self) -> &mut Controller {
        &mut self.devices.devices.controller1
    }
    pub fn mapper(&self) -> &M {
        &self.devices.devices.mapper
    }
    pub fn devices_with_oam(&self) -> &NesDevicesWithOam<M> {
        &self.devices
    }
}

pub mod controller1 {
    use super::*;
    pub mod press {
        use super::*;
        pub fn left<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_left();
        }
        pub fn right<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_right();
        }
        pub fn up<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_up();
        }
        pub fn down<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_down();
        }
        pub fn start<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_start();
        }
        pub fn select<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_select();
        }
        pub fn a<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_a();
        }
        pub fn b<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.set_b();
        }
    }
    pub mod release {
        use super::*;
        pub fn left<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_left();
        }
        pub fn right<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_right();
        }
        pub fn up<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_up();
        }
        pub fn down<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_down();
        }
        pub fn start<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_start();
        }
        pub fn select<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_select();
        }
        pub fn a<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_a();
        }
        pub fn b<M: Mapper>(nes: &mut Nes<M>) {
            nes.devices.devices.controller1.clear_b();
        }
    }
}
