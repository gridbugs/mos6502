/// Load and store values exercising all addressing modes.
/// Populates the first 24 bytes of memory with
/// 0xAA x 4
/// 0xBB x 4
/// 0xCC x 4
/// 0xDD x 4
/// 0xEE x 4
/// 0xFF x 4
///
/// Instructions:
/// - JMP (Absolute)
/// - LDX (Immediate)
/// - LDY (Immediate)
/// - LDA (Immediate)
/// - LDA (ZeroPage)
/// - LDA (ZeroPageXIndexed)
/// - LDA (Absolute)
/// - LDA (AbsoluteXIndexed)
/// - LDA (AbsoluteYIndexed)
/// - LDA (XIndexedIndirect)
/// - LDA (IndirectYIndexed)
/// - STA (ZeroPage)
/// - STA (ZeroPageXIndexed)
/// - STA (Absolute)
/// - STA (AbsoluteXIndexed)
/// - STA (AbsoluteYIndexed)
/// - STA (XIndexedIndirect)
/// - STA (IndirectYIndexed)
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
        b.inst(Lda(Immediate), 0xAA);
        b.inst(Sta(ZeroPage), 0);
        b.inst(Sta(ZeroPage), 1);
        b.inst(Sta(ZeroPage), 2);
        b.inst(Sta(ZeroPage), 3);

        b.inst(Lda(Immediate), 0xBB);
        b.inst(Ldx(Immediate), 0);
        b.inst(Sta(ZeroPageXIndexed), 4);
        b.inst(Ldx(Immediate), 1);
        b.inst(Sta(ZeroPageXIndexed), 4);
        b.inst(Ldx(Immediate), 2);
        b.inst(Sta(ZeroPageXIndexed), 4);
        b.inst(Ldx(Immediate), 0xFF);
        b.inst(Sta(ZeroPageXIndexed), 8);

        b.inst(Lda(Absolute), "src0");
        b.inst(Sta(Absolute), 0x0008u16);
        b.inst(Sta(Absolute), 0x0009u16);
        b.inst(Sta(Absolute), 0x000Au16);
        b.inst(Sta(Absolute), 0x000Bu16);
        b.inst(Ldx(Immediate), 1);

        b.inst(Lda(AbsoluteXIndexed), "src0");
        b.inst(Ldx(Immediate), 0);
        b.inst(Sta(AbsoluteXIndexed), 0x000Cu16);
        b.inst(Ldy(Immediate), 1);
        b.inst(Sta(AbsoluteYIndexed), 0x000Cu16);
        b.inst(Ldx(Immediate), 2);
        b.inst(Sta(AbsoluteXIndexed), 0x000Cu16);
        b.inst(Ldy(Immediate), 3);
        b.inst(Sta(AbsoluteYIndexed), 0x000Cu16);

        // Prepare for indexed indirect and indirect indexed accesses by writing some addresses
        // into the zero page.

        b.inst(Lda(Immediate), LabelOffsetLo("src1"));
        b.inst(Sta(ZeroPage), 0x20);
        b.inst(Lda(Immediate), LabelOffsetHi("src1"));
        b.inst(Sta(ZeroPage), 0x21);
        b.inst(Lda(Immediate), LabelOffsetLo("src0"));
        b.inst(Sta(ZeroPage), 0x22);
        b.inst(Lda(Immediate), LabelOffsetHi("src0"));
        b.inst(Sta(ZeroPage), 0x23);
        b.inst(Lda(Immediate), 0x10);
        b.inst(Sta(ZeroPage), 0x30);
        b.inst(Lda(Immediate), 0x00);
        b.inst(Sta(ZeroPage), 0x31);
        b.inst(Lda(Immediate), 0x12);
        b.inst(Sta(ZeroPage), 0x32);
        b.inst(Lda(Immediate), 0x00);
        b.inst(Sta(ZeroPage), 0x33);
        b.inst(Lda(Immediate), 0x13);
        b.inst(Sta(ZeroPage), 0x34);
        b.inst(Lda(Immediate), 0x00);
        b.inst(Sta(ZeroPage), 0x35);

        // The address of "src1" is now located at 0x0020.
        // The address of "src0" is now located at 0x0022.
        // The zero page address 0x0010 is now located at 0x0030.
        // The zero page address 0x0012 is now located at 0x0032.
        // The zero page address 0x0013 is now located at 0x0033.

        b.inst(Ldx(Immediate), 8);
        b.inst(Lda(XIndexedIndirect), 0x18);
        b.inst(Ldy(Immediate), 0);
        b.inst(Sta(IndirectYIndexed), 0x30);
        b.inst(Ldy(Immediate), 1);
        b.inst(Sta(IndirectYIndexed), 0x30);
        b.inst(Lda(Immediate), 0);
        b.inst(Ldy(Immediate), 2);
        b.inst(Lda(IndirectYIndexed), 0x22);
        b.inst(Ldx(Immediate), 2);
        b.inst(Sta(XIndexedIndirect), 0x30);
        b.inst(Ldx(Immediate), 4);
        b.inst(Sta(XIndexedIndirect), 0x30);

        b.inst(Ldy(Immediate), 1);
        b.inst(Lda(IndirectYIndexed), 0x20);
        b.inst(Sta(Absolute), 0x0014u16);
        b.inst(Lda(Immediate), 0);
        b.inst(Ldx(Immediate), 0x14);
        b.inst(Lda(ZeroPageXIndexed), 0);
        b.inst(Sta(ZeroPageXIndexed), 1);
        b.inst(Sta(ZeroPageXIndexed), 2);
        b.inst(Sta(ZeroPageXIndexed), 3);

        // Loop forever
        b.label("loop");
        b.inst(Jmp(Absolute), "loop");

        // Some data that this program will use
        b.label("src0");
        b.literal_byte(0xCC);
        b.literal_byte(0xDD);
        b.label("src1");
        b.literal_byte(0xEE);
        b.literal_byte(0xFF);
    });
}
