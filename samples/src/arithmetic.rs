/// Perform addition, storing the resultant value and status register in the zero
/// page.  For reference, the status register bits have the following meaning:
/// N V _ B D I Z C
/// (bit 5 is unused and is often 1 when read).
///
/// Note that SBC treats the carry flag as "not borrowed". Carry must be set to 1
/// before performing subtraction unless a borrow is required, and "carry == 1"
/// after subtraction means that no borrow was performed.
use crate::prelude::*;

pub struct Arithmetic;
impl Sample for Arithmetic {
    fn program(b: &mut Block) {
        #[derive(Default)]
        struct StoreResult {
            offset: u8,
        }
        impl StoreResult {
            fn store(&mut self, b: &mut Block) {
                b.inst(Sta(ZeroPage), self.offset);
                b.inst(Php, ());
                b.inst(Pla, ());
                b.inst(Sta(ZeroPage), self.offset + 1);
                self.offset += 2;
            }
        }

        let mut store_result = StoreResult::default();

        b.inst(Cld, ());

        // 0
        // 5 + 7 = 12 (0x0C)
        // flags:
        b.inst(Clc, ());
        b.inst(Lda(Immediate), 5);
        b.inst(Adc(Immediate), 7);
        store_result.store(b);

        // 1
        // (+127) + (+2) = -127 (0x81)
        // flags: NV
        b.inst(Clc, ());
        b.inst(Lda(Immediate), 127);
        b.inst(Adc(Immediate), 2);
        store_result.store(b);

        // 2
        // 13 + 211 + 1 = 225 (0xE1)
        // flags: N
        b.inst(Sec, ());
        b.inst(Lda(Immediate), 13);
        b.inst(Adc(Immediate), 211);
        store_result.store(b);

        // 3
        // 254 + 6 + 1 = 5
        // flags: C
        b.inst(Sec, ());
        b.inst(Lda(Immediate), 254);
        b.inst(Adc(Immediate), 6);
        store_result.store(b);

        // 4
        // (+5) + (-3) = 2
        // flags: C
        b.inst(Clc, ());
        b.inst(Lda(Immediate), 5);
        b.inst(Adc(Immediate), -3);
        store_result.store(b);

        // 5
        // (-5) + (-7) = -12 (0xF4)
        // flags: NC
        b.inst(Clc, ());
        b.inst(Lda(Immediate), -5);
        b.inst(Adc(Immediate), -7);
        store_result.store(b);

        // 6
        // (-66) + (-65) = -131, underflows to +125
        // flags: CV
        b.inst(Clc, ());
        b.inst(Lda(Immediate), -66);
        b.inst(Adc(Immediate), -65);
        store_result.store(b);

        // 7
        // 5 - 3 = 2
        // flags: C
        b.inst(Sec, ());
        b.inst(Lda(Immediate), 5);
        b.inst(Sbc(Immediate), 3);
        store_result.store(b);

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    }
    fn num_steps() -> usize {
        100
    }
    fn check_result<M: MemoryReadOnly>(_cpu: &Cpu, m: &M) {
        use status::flag::*;
        #[derive(Default)]
        struct CheckResult {
            offset: Address,
        }
        impl CheckResult {
            fn check<M: MemoryReadOnly>(&mut self, m: &M, result: u8, status: u8) {
                assert_eq!(m.read_u8_read_only(self.offset), result);
                assert_eq!(m.read_u8_read_only(self.offset + 1), EXPANSION | status);
                self.offset += 2;
            }
        }

        let mut check_result = CheckResult::default();

        // 0
        check_result.check(m, 0x0C, 0);

        // 1
        check_result.check(m, 0x81, NEGATIVE | OVERFLOW);

        // 2
        check_result.check(m, 0xE1, NEGATIVE);

        // 3
        check_result.check(m, 0x05, CARRY);

        // 4
        check_result.check(m, 0x02, CARRY);

        // 5
        check_result.check(m, 0xF4, NEGATIVE | CARRY);

        // 6
        check_result.check(m, 125, CARRY | OVERFLOW);

        // 7
        check_result.check(m, 2, CARRY);
    }
}
