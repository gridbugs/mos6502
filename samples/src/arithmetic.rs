/// Perform addition, storing the resultant value and status register in the zero
/// page.  For reference, the status register bits have the following meaning:
/// N V _ B D I Z C
/// (bit 5 is unused and is often 1 when read).
///
use crate::prelude::*;

pub struct Arithmetic;
impl Sample for Arithmetic {
    fn program(b: &mut Block) {
        fn store_result(b: &mut Block, offset: u8) {
            b.inst(Sta(ZeroPage), offset);
            b.inst(Php, ());
            b.inst(Pla, ());
            b.inst(Sta(ZeroPage), offset + 1);
        }

        let mut offset = 0;
        b.inst(Cld, ());

        // 5 + 7 = 12 (0x0C)
        // flags:
        b.inst(Clc, ());
        b.inst(Lda(Immediate), 5);
        b.inst(Adc(Immediate), 7);
        store_result(b, offset);
        offset += 2;

        // (+127) + (+2) = -127 (0x81)
        // flags: NV
        b.inst(Clc, ());
        b.inst(Lda(Immediate), 127);
        b.inst(Adc(Immediate), 2);
        store_result(b, offset);
        offset += 2;

        // 13 + 211 + 1 = 225 (0xE1)
        // flags: N
        b.inst(Sec, ());
        b.inst(Lda(Immediate), 13);
        b.inst(Adc(Immediate), 211);
        store_result(b, offset);
        offset += 2;

        // 254 + 6 + 1 = 5
        // flags: C
        b.inst(Sec, ());
        b.inst(Lda(Immediate), 254);
        b.inst(Adc(Immediate), 6);
        store_result(b, offset);
        offset += 2;

        // (+5) + (-3) = 2
        // flags: C
        b.inst(Clc, ());
        b.inst(Lda(Immediate), 5);
        b.inst(Adc(Immediate), -3);
        store_result(b, offset);

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    }
    fn num_steps() -> usize {
        100
    }
    fn check_result<M: MemoryReadOnly>(_cpu: &Cpu, m: &M) {
        assert_eq!(m.read_u8_read_only(0), 0x0C);
        assert_eq!(m.read_u8_read_only(1), 0x20); // assume bit 5 is set
        assert_eq!(m.read_u8_read_only(2), 0x81);
        assert_eq!(m.read_u8_read_only(3), 0xE0);
        assert_eq!(m.read_u8_read_only(4), 0xE1);
        assert_eq!(m.read_u8_read_only(5), 0xA0);
        assert_eq!(m.read_u8_read_only(6), 0x05);
        assert_eq!(m.read_u8_read_only(7), 0x21);
        assert_eq!(m.read_u8_read_only(8), 0x02);
        assert_eq!(m.read_u8_read_only(9), 0x21);
    }
}
