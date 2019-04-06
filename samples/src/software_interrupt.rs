/// Example program using the BRK instruction to trigger an interrupt.
/// The interrupt handler checks that the BRK bit is set in the status
/// pushed onto the stack during the interrupt.
///
/// After running, the first 4 bytes of memory should be 0xAA, and the
/// next 4 bytes should be 0xBB.
///
/// Intructions:
/// BRK
/// RTI
/// LDX
/// LDA
/// STA
/// TAX
/// TXA
/// PLA
/// PHA
/// BEQ
use crate::prelude::*;

pub struct SoftwareInterrupt;
impl Sample for SoftwareInterrupt {
    fn program(b: &mut Block) {
        b.inst(Lda(Immediate), 0xAA);
        b.inst(Sta(Absolute), 0u16);
        b.inst(Sta(Absolute), 1u16);
        b.inst(Sta(Absolute), 2u16);
        b.inst(Sta(Absolute), 3u16);
        b.inst(Brk, ());
        b.literal_byte(0xFF);
        b.inst(Sta(Absolute), 4u16);
        b.inst(Sta(Absolute), 5u16);
        b.inst(Sta(Absolute), 6u16);
        b.inst(Sta(Absolute), 7u16);

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");

        b.label("irq_handler");
        b.inst(Tax, ());
        b.inst(Pla, ());
        b.inst(Pha, ());
        b.inst(And(Immediate), status::flag::BRK);
        b.inst(Beq, LabelRelativeOffset("irq_handler_a"));
        b.inst(Ldx(Immediate), 0xBB);
        b.label("irq_handler_a");
        b.inst(Txa, ());
        b.inst(Rti, ());

        b.set_offset(interrupt_vector::IRQ_LO - PRG_START);
        b.label_offset_le("irq_handler");
    }
    fn num_steps() -> usize {
        1000
    }
    fn check_result<M: MemoryReadOnly>(_: &Cpu, m: &M) {
        for i in 0..4 {
            assert_eq!(m.read_u8_read_only(i), 0xAA);
        }
        for i in 4..8 {
            assert_eq!(m.read_u8_read_only(i), 0xBB);
        }
    }
}
