extern crate mos6502;

use mos6502::debug::{AddressingMode, Instruction, InstructionType, InstructionWithOperand};
use mos6502::machine::MemoryReadOnly;
use mos6502::opcode;
use mos6502::Address;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub trait MemoryMap {
    fn normalize_function_call<M: MemoryReadOnly>(
        &self,
        jsr_opcode_address: Address,
        memory: &M,
    ) -> Option<Address>;
}

fn enumerate_function_definition_addresses<MRO: MemoryReadOnly, MM: MemoryMap>(
    memory: &MRO,
    memory_map: &MM,
) -> Vec<Address> {
    let mut function_definition_addresses = BTreeSet::new();
    for address in 0..=0xFFFF {
        let byte = memory.read_u8_read_only(address);
        if byte == opcode::jsr::ABSOLUTE {
            if let Some(function_address) = memory_map.normalize_function_call(address, memory) {
                function_definition_addresses.insert(function_address);
            }
        }
    }
    let mut function_definition_addresses = function_definition_addresses
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    function_definition_addresses.sort();
    function_definition_addresses
}

#[derive(Debug)]
enum FunctionStep {
    InvalidOpcode {
        address: Address,
        opcode: u8,
    },
    JumpIndirect(InstructionWithOperand),
    TracedInstruction(InstructionWithOperand),
    FunctionCall {
        instruction_with_operand: InstructionWithOperand,
        callee: Address,
    },
    Branch {
        instruction_with_operand: InstructionWithOperand,
        absolute_target: Address,
        relative_target: i8,
    },
}

impl fmt::Display for FunctionStep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionStep::TracedInstruction(instruction_with_operand)
            | FunctionStep::FunctionCall {
                instruction_with_operand,
                ..
            } => instruction_with_operand.fmt(f),
            FunctionStep::JumpIndirect(instruction_with_operand) => {
                write!(f, "{} <--- END", instruction_with_operand)
            }
            FunctionStep::Branch {
                instruction_with_operand,
                absolute_target,
                relative_target,
            } => write!(
                f,
                "{} (relative: {}, absolute: {:X})",
                instruction_with_operand, relative_target, absolute_target
            ),
            FunctionStep::InvalidOpcode { address, opcode } => {
                write!(f, "{:04X}  ???????? ({:02X})", address, opcode)
            }
        }
    }
}

impl FunctionStep {
    fn address(&self) -> Address {
        match self {
            FunctionStep::InvalidOpcode { address, .. } => *address,
            FunctionStep::JumpIndirect(instruction_with_operand)
            | FunctionStep::TracedInstruction(instruction_with_operand)
            | FunctionStep::FunctionCall {
                instruction_with_operand,
                ..
            }
            | FunctionStep::Branch {
                instruction_with_operand,
                ..
            } => instruction_with_operand.address(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionTrace {
    steps: Vec<FunctionStep>,
}

impl fmt::Display for FunctionTrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for step in self.steps.iter() {
            writeln!(f, "{}", step)?;
        }
        Ok(())
    }
}
impl FunctionTrace {
    fn steps(&self) -> &[FunctionStep] {
        &self.steps
    }
    fn len(&self) -> usize {
        self.steps.len()
    }
    fn contains_address(&self, address: Address) -> bool {
        for step in self.steps.iter() {
            if step.address() == address {
                return true;
            }
        }
        false
    }
}

fn trace_function_definition<M: MemoryReadOnly>(
    function_definition_address: Address,
    memory: &M,
) -> FunctionTrace {
    let mut steps = Vec::new();
    let mut seen = BTreeSet::new();
    let mut to_visit = vec![function_definition_address];
    while let Some(visited_address) = to_visit.pop() {
        if seen.contains(&visited_address) {
            continue;
        }
        seen.insert(visited_address);
        if let Ok(instruction_with_operand) =
            InstructionWithOperand::decode(visited_address, memory)
        {
            let instruction = instruction_with_operand.instruction();
            match instruction.instruction_type() {
                InstructionType::Jmp => match instruction.addressing_mode() {
                    AddressingMode::Absolute => {
                        let address = memory.read_u16_le_read_only(visited_address.wrapping_add(1));
                        to_visit.push(address);
                    }
                    AddressingMode::Indirect => {
                        steps.push(FunctionStep::JumpIndirect(instruction_with_operand));
                        continue;
                    }
                    _ => panic!("Invalid addressing mode"),
                },
                InstructionType::Rti => (),
                InstructionType::Bcc
                | InstructionType::Beq
                | InstructionType::Bmi
                | InstructionType::Bne
                | InstructionType::Bpl
                | InstructionType::Bvc
                | InstructionType::Bvs => {
                    let offset = memory.read_u8_read_only(visited_address.wrapping_add(1)) as i8;
                    let absolute_target = ((visited_address + instruction.size() as Address) as i16
                        + offset as i16) as Address;
                    to_visit.push(absolute_target);
                    let next_address = visited_address.wrapping_add(instruction.size() as Address);
                    to_visit.push(next_address);
                    steps.push(FunctionStep::Branch {
                        instruction_with_operand,
                        absolute_target,
                        relative_target: offset,
                    });
                    continue;
                }
                other => {
                    let next_address = visited_address.wrapping_add(instruction.size() as Address);
                    to_visit.push(next_address);
                    if let InstructionType::Jsr = other {
                        steps.push(FunctionStep::FunctionCall {
                            callee: instruction_with_operand.operand_u16_le().unwrap(),
                            instruction_with_operand,
                        });
                        continue;
                    }
                }
            }
            steps.push(FunctionStep::TracedInstruction(instruction_with_operand));
        } else {
            steps.push(FunctionStep::InvalidOpcode {
                address: visited_address,
                opcode: memory.read_u8_read_only(visited_address),
            });
        }
    }
    FunctionTrace { steps }
}

#[derive(Debug)]
struct CallGraph {
    by_caller: BTreeMap<Address, BTreeSet<Address>>,
    by_callee: BTreeMap<Address, BTreeSet<Address>>,
}

impl CallGraph {
    fn new() -> Self {
        Self {
            by_caller: BTreeMap::new(),
            by_callee: BTreeMap::new(),
        }
    }
    fn insert(&mut self, caller: Address, callee: Address) {
        self.by_caller
            .entry(caller)
            .or_insert_with(BTreeSet::new)
            .insert(callee);
        self.by_callee
            .entry(callee)
            .or_insert_with(BTreeSet::new)
            .insert(caller);
    }
}

pub struct Analysis {
    call_graph: CallGraph,
    function_traces_by_definition_address: BTreeMap<Address, FunctionTrace>,
}

impl Analysis {
    pub fn analyse<MRO: MemoryReadOnly, MM: MemoryMap>(memory: &MRO, memory_map: &MM) -> Self {
        let function_definition_addresses =
            enumerate_function_definition_addresses(memory, memory_map);
        let mut function_traces_by_definition_address = BTreeMap::new();
        let mut call_graph = CallGraph::new();
        for &function_definition_address in function_definition_addresses.iter() {
            let trace = trace_function_definition(function_definition_address, memory);
            for step in trace.steps() {
                if let FunctionStep::FunctionCall { callee, .. } = step {
                    call_graph.insert(function_definition_address, *callee);
                }
            }
            function_traces_by_definition_address.insert(function_definition_address, trace);
        }
        Self {
            call_graph,
            function_traces_by_definition_address,
        }
    }
    pub fn function_trace(&self, address: Address) -> Option<&FunctionTrace> {
        self.function_traces_by_definition_address.get(&address)
    }
    pub fn functions_containing_address<'a>(
        &'a self,
        address: Address,
    ) -> impl 'a + Iterator<Item = Address> {
        self.function_traces_by_definition_address
            .iter()
            .filter_map(move |(definition_address, trace)| {
                if trace.contains_address(address) {
                    Some(*definition_address)
                } else {
                    None
                }
            })
    }
}
