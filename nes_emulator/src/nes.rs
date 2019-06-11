use crate::apu::Apu;
use crate::mapper::Mapper;
use crate::ppu::{Oam, Ppu, RenderOutput};
use crate::timing;
use crate::DynamicNes;
use mos6502::debug::InstructionWithOperand;
use mos6502::machine::{Address, Cpu, Memory, MemoryReadOnly};
use nes_specs;
use std::io::{self, Write};

const RAM_BYTES: usize = 0x800;

big_array! { BigArray; }

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
struct NesDevicesWithOam<M: Mapper> {
    devices: NesDevices<M>,
    oam: Oam,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

trait RunForCycles {
    fn run_for_cycles<M: Memory + MemoryReadOnly>(cpu: &mut Cpu, memory: &mut M, num_cycles: u32);
}

struct RunForCyclesRegular;
struct RunForCyclesDebug;

impl RunForCycles for RunForCyclesRegular {
    fn run_for_cycles<M: Memory + MemoryReadOnly>(cpu: &mut Cpu, memory: &mut M, num_cycles: u32) {
        cpu.run_for_cycles(memory, num_cycles as usize).unwrap();
    }
}

impl RunForCycles for RunForCyclesDebug {
    fn run_for_cycles<M: Memory + MemoryReadOnly>(cpu: &mut Cpu, memory: &mut M, num_cycles: u32) {
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
    pub fn render<R: RenderOutput>(&self, render_output: &mut R) {
        self.devices.devices.ppu.render(
            &self.devices.devices.mapper,
            &self.devices.oam,
            render_output,
        );
    }
    fn run_for_frame_general<R: RunForCycles>(&mut self, _: R) {
        // pre-render scanline
        R::run_for_cycles(
            &mut self.cpu,
            &mut self.devices,
            timing::ntsc::APPROX_CPU_CYCLES_PER_SCANLINE,
        );
        for _ in 0..nes_specs::SCREEN_HEIGHT_PX {
            R::run_for_cycles(
                &mut self.cpu,
                &mut self.devices,
                timing::ntsc::APPROX_CPU_CYCLES_PER_SCANLINE,
            );
        }
        // post-render scanline
        R::run_for_cycles(
            &mut self.cpu,
            &mut self.devices,
            timing::ntsc::APPROX_CPU_CYCLES_PER_SCANLINE,
        );
        if self.devices.devices.ppu.is_vblank_nmi_enabled() {
            self.cpu.nmi(&mut self.devices);
        }
        self.devices.devices.ppu.set_vblank();
        R::run_for_cycles(
            &mut self.cpu,
            &mut self.devices,
            timing::ntsc::APPROX_CPU_CYCLES_PER_VBLANK,
        );
        self.devices.devices.ppu.clear_vblank();
    }
    pub fn run_for_frame(&mut self) {
        self.run_for_frame_general(RunForCyclesRegular);
    }
    pub fn run_for_frame_debug(&mut self) {
        self.run_for_frame_general(RunForCyclesDebug);
    }
    pub fn clone_dynamic_nes(&self) -> DynamicNes {
        M::clone_dynamic_nes(self)
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
