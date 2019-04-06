/// Compute the factorial of 5 and store it in the first byte of memory by calling a recursive
/// function.
///
/// Instructions:
/// Jsr
/// Rts
/// Lda
/// Sta
/// Ldx
/// Ldy
/// Adc
/// Sbc
/// Inx
/// Dex
/// Clc
/// Sec
/// Cmp
/// Bcs
/// Beq
/// Jmp
///
/// ## Function Calling Convention
///
/// This program uses a second "stack" in the zero page (0x0000..0x00FF) for storing function
/// arguments and return values. Upon calling a function, the X index register will point 1 after
/// the last argument.  Upon returning from a function, the X index register will point 1 after the
/// return value. This makes it easy to pass the result of one function directly as an argument to
/// another function.
use crate::prelude::*;

pub struct Factorial;
impl Sample for Factorial {
    fn program(b: &mut Block) {
        b.inst(Ldx(Immediate), 0);

        b.inst(Lda(Immediate), 5);
        b.inst(Sta(ZeroPageXIndexed), 0);
        b.inst(Inx, ());
        b.inst(Jsr(Absolute), "factorial");

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");

        b.label("multiply");
        b.inst(Lda(Immediate), 0);
        b.inst(Dex, ());
        b.label("multiply_a");
        b.inst(Dex, ());
        b.inst(Ldy(ZeroPageXIndexed), 0);
        b.inst(Beq, LabelRelativeOffset("multiply_b"));
        b.inst(Dec(ZeroPageXIndexed), 0);
        b.inst(Inx, ());
        b.inst(Clc, ());
        b.inst(Adc(ZeroPageXIndexed), 0);
        b.inst(Jmp(Absolute), "multiply_a");
        b.label("multiply_b");
        b.inst(Sta(ZeroPageXIndexed), 0);
        b.inst(Inx, ());
        b.inst(Rts, ());

        b.label("factorial");
        b.inst(Dex, ());
        b.inst(Lda(Immediate), 1);
        b.inst(Cmp(ZeroPageXIndexed), 0);
        b.inst(Bcs, LabelRelativeOffset("factorial_a"));
        b.inst(Lda(ZeroPageXIndexed), 0);
        b.inst(Sec, ());
        b.inst(Sbc(Immediate), 1);
        b.inst(Inx, ());
        b.inst(Sta(ZeroPageXIndexed), 0);
        b.inst(Inx, ());
        b.inst(Jsr(Absolute), "factorial");
        b.inst(Jsr(Absolute), "multiply");
        b.inst(Rts, ());
        b.label("factorial_a");
        b.inst(Sta(ZeroPageXIndexed), 0);
        b.inst(Inx, ());
        b.inst(Rts, ());
    }
    fn num_steps() -> usize {
        10000
    }
    fn check_result<M: MemoryReadOnly>(_: &Cpu, m: &M) {
        assert_eq!(m.read_u8_read_only(0), 120);
    }
}
