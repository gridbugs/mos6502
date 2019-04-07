/// Compute the factorial of 12 and store it little endian in the first 4 bytes of memory.
/// Demonstrates how rust can be used as a macro language for 6502 assembly programs.
///
/// Instructions:
/// JSR
/// RTS
/// LDA
/// STA
/// LDX
/// LDY
/// STY
/// INX
/// DEX
/// ADC
/// SBC
/// BNE
/// JMP
///
/// Uses the same calling convention as the "factorial" sample.
use crate::prelude::*;

const ARG: u32 = 12;

pub struct WideFactorial;
impl Sample for WideFactorial {
    fn program(b: &mut Block) {
        fn push_u32(b: &mut Block, value: u32) {
            b.inst(Ldy(Immediate), value as u8);
            b.inst(Sty(ZeroPageXIndexed), 0);
            b.inst(Inx, ());
            b.inst(Ldy(Immediate), (value.wrapping_shr(8)) as u8);
            b.inst(Sty(ZeroPageXIndexed), 0);
            b.inst(Inx, ());
            b.inst(Ldy(Immediate), (value.wrapping_shr(16)) as u8);
            b.inst(Sty(ZeroPageXIndexed), 0);
            b.inst(Inx, ());
            b.inst(Ldy(Immediate), (value.wrapping_shr(24)) as u8);
            b.inst(Sty(ZeroPageXIndexed), 0);
            b.inst(Inx, ());
        }

        fn call(b: &mut Block, label: &'static str) {
            b.inst(Jsr(Absolute), label);
        }

        b.inst(Ldx(Immediate), 0);

        push_u32(b, ARG);
        call(b, "factorial_32");

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");

        b.label("add_32");
        b.inst(Clc, ());
        for _ in 0..4 {
            b.inst(Dex, ());
        }
        for i in 0..4 {
            b.inst(Lda(ZeroPageXIndexed), i - 4);
            b.inst(Adc(ZeroPageXIndexed), i);
            b.inst(Sta(ZeroPageXIndexed), i - 4);
        }
        b.inst(Rts, ());

        b.label("decrement_32");
        b.inst(Sec, ());
        b.inst(Lda(ZeroPageXIndexed), -4);
        b.inst(Sbc(Immediate), 1);
        b.inst(Sta(ZeroPageXIndexed), -4);
        for i in 1..4 {
            b.inst(Lda(ZeroPageXIndexed), i - 4);
            b.inst(Sbc(Immediate), 0);
            b.inst(Sta(ZeroPageXIndexed), i - 4);
        }
        b.inst(Rts, ());

        b.label("multiply_32");
        b.inst(Ldy(Immediate), 0);
        for i in 0..4 {
            b.inst(Lda(ZeroPageXIndexed), i - 4);
            b.inst(Sty(ZeroPageXIndexed), i - 4);
            b.inst(Sta(ZeroPageXIndexed), i);
        }
        b.label("multiply_32_next");
        for i in 0..4 {
            b.inst(Lda(ZeroPageXIndexed), i - 8);
            b.inst(Bne, LabelRelativeOffset("multiply_32_non_zero"));
        }
        for _ in 0..4 {
            b.inst(Dex, ());
        }
        for i in 0..4 {
            b.inst(Lda(ZeroPageXIndexed), i);
            b.inst(Sta(ZeroPageXIndexed), i - 4);
        }
        b.inst(Rts, ());
        b.label("multiply_32_non_zero");
        for _ in 0..4 {
            b.inst(Inx, ());
        }
        call(b, "add_32");
        for _ in 0..4 {
            b.inst(Dex, ());
        }
        call(b, "decrement_32");
        for _ in 0..4 {
            b.inst(Inx, ());
        }
        b.inst(Jmp(Absolute), "multiply_32_next");

        b.label("factorial_32");
        for i in 0..4 {
            b.inst(Lda(ZeroPageXIndexed), i - 4);
            b.inst(Bne, LabelRelativeOffset("factorial_32_non_zero"));
        }
        b.inst(Inc(ZeroPageXIndexed), -4);
        b.inst(Rts, ());
        b.label("factorial_32_non_zero");
        for i in 0..4 {
            b.inst(Lda(ZeroPageXIndexed), i - 4);
            b.inst(Sta(ZeroPageXIndexed), i);
        }
        for _ in 0..4 {
            b.inst(Inx, ());
        }
        call(b, "decrement_32");
        call(b, "factorial_32");
        call(b, "multiply_32");
        b.inst(Rts, ());
    }
    fn num_steps() -> usize {
        1000000
    }
    fn check_result<M: MemoryReadOnly>(_: &Cpu, m: &M) {
        fn factorial(n: u32) -> u32 {
            match n {
                0 => 1,
                n => n * factorial(n - 1),
            }
        }
        let result = factorial(ARG);
        assert_eq!(m.read_u8_read_only(0), result.wrapping_shr(0) as u8);
        assert_eq!(m.read_u8_read_only(1), result.wrapping_shr(8) as u8);
        assert_eq!(m.read_u8_read_only(2), result.wrapping_shr(16) as u8);
        assert_eq!(m.read_u8_read_only(3), result.wrapping_shr(24) as u8);
    }
}
