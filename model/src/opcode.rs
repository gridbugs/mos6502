pub mod adc {
    pub const ABSOLUTE: u8 = 0x6D;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x7D;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0x79;
    pub const IMMEDIATE: u8 = 0x69;
    pub const INDIRECT_Y_INDEXED: u8 = 0x71;
    pub const X_INDEXED_INDIRECT: u8 = 0x61;
    pub const ZERO_PAGE: u8 = 0x65;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x75;
}
pub mod ahx {
    pub mod unofficial0 {
        pub const INDIRECT_Y_INDEXED: u8 = 0x93;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0x9F;
    }
}
pub mod anc {
    pub mod unofficial0 {
        pub const IMMEDIATE: u8 = 0x0B;
    }
    pub mod unofficial1 {
        pub const IMMEDIATE: u8 = 0x2B;
    }
}
pub mod and {
    pub const ABSOLUTE: u8 = 0x2D;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x3D;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0x39;
    pub const IMMEDIATE: u8 = 0x29;
    pub const INDIRECT_Y_INDEXED: u8 = 0x31;
    pub const X_INDEXED_INDIRECT: u8 = 0x21;
    pub const ZERO_PAGE: u8 = 0x25;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x35;
}
pub mod alr {
    pub mod unofficial0 {
        pub const IMMEDIATE: u8 = 0x4B;
    }
}
pub mod arr {
    pub mod unofficial0 {
        pub const IMMEDIATE: u8 = 0x6B;
    }
}
pub mod asl {
    pub const ABSOLUTE: u8 = 0x0E;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x1E;
    pub const ACCUMULATOR: u8 = 0x0A;
    pub const ZERO_PAGE: u8 = 0x06;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x16;
}
pub mod axs {
    pub mod unofficial0 {
        pub const IMMEDIATE: u8 = 0xCB;
    }
}
pub mod bcc {
    pub const RELATIVE: u8 = 0x90;
}
pub mod bcs {
    pub const RELATIVE: u8 = 0xB0;
}
pub mod beq {
    pub const RELATIVE: u8 = 0xF0;
}
pub mod bmi {
    pub const RELATIVE: u8 = 0x30;
}
pub mod bne {
    pub const RELATIVE: u8 = 0xD0;
}
pub mod bpl {
    pub const RELATIVE: u8 = 0x10;
}
pub mod brk {
    pub const IMPLIED: u8 = 0x00;
}
pub mod bvc {
    pub const RELATIVE: u8 = 0x50;
}
pub mod bvs {
    pub const RELATIVE: u8 = 0x70;
}
pub mod bit {
    pub const ZERO_PAGE: u8 = 0x24;
    pub const ABSOLUTE: u8 = 0x2C;
}
pub mod clc {
    pub const IMPLIED: u8 = 0x18;
}
pub mod cld {
    pub const IMPLIED: u8 = 0xD8;
}
pub mod cli {
    pub const IMPLIED: u8 = 0x58;
}
pub mod clv {
    pub const IMPLIED: u8 = 0xB8;
}
pub mod cmp {
    pub const ABSOLUTE: u8 = 0xCD;
    pub const ABSOLUTE_X_INDEXED: u8 = 0xDD;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0xD9;
    pub const IMMEDIATE: u8 = 0xC9;
    pub const INDIRECT_Y_INDEXED: u8 = 0xD1;
    pub const X_INDEXED_INDIRECT: u8 = 0xC1;
    pub const ZERO_PAGE: u8 = 0xC5;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0xD5;
}
pub mod dcp {
    pub mod unofficial0 {
        pub const X_INDEXED_INDIRECT: u8 = 0xC3;
        pub const ZERO_PAGE: u8 = 0xC7;
        pub const ABSOLUTE: u8 = 0xCF;
        pub const INDIRECT_Y_INDEXED: u8 = 0xD3;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0xD7;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0xDB;
        pub const ABSOLUTE_X_INDEXED: u8 = 0xDF;
    }
}
pub mod dec {
    pub const ABSOLUTE: u8 = 0xCE;
    pub const ABSOLUTE_X_INDEXED: u8 = 0xDE;
    pub const ZERO_PAGE: u8 = 0xC6;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0xD6;
}
pub mod cpx {
    pub const ABSOLUTE: u8 = 0xEC;
    pub const IMMEDIATE: u8 = 0xE0;
    pub const ZERO_PAGE: u8 = 0xE4;
}
pub mod cpy {
    pub const ABSOLUTE: u8 = 0xCC;
    pub const IMMEDIATE: u8 = 0xC0;
    pub const ZERO_PAGE: u8 = 0xC4;
}
pub mod dex {
    pub const IMPLIED: u8 = 0xCA;
}
pub mod dey {
    pub const IMPLIED: u8 = 0x88;
}
pub mod eor {
    pub const ABSOLUTE: u8 = 0x4D;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x5D;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0x59;
    pub const IMMEDIATE: u8 = 0x49;
    pub const INDIRECT_Y_INDEXED: u8 = 0x51;
    pub const X_INDEXED_INDIRECT: u8 = 0x41;
    pub const ZERO_PAGE: u8 = 0x45;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x55;
}
pub mod ign {
    pub mod unofficial0 {
        pub const ABSOLUTE: u8 = 0x0C;
        pub const ABSOLUTE_X_INDEXED: u8 = 0x1C;
        pub const ZERO_PAGE: u8 = 0x04;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x14;
    }
    pub mod unofficial1 {
        pub const ABSOLUTE_X_INDEXED: u8 = 0x3C;
        pub const ZERO_PAGE: u8 = 0x44;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x34;
    }
    pub mod unofficial2 {
        pub const ABSOLUTE_X_INDEXED: u8 = 0x5C;
        pub const ZERO_PAGE: u8 = 0x64;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x54;
    }
    pub mod unofficial3 {
        pub const ABSOLUTE_X_INDEXED: u8 = 0x7C;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x74;
    }
    pub mod unofficial4 {
        pub const ABSOLUTE_X_INDEXED: u8 = 0xDC;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0xD4;
    }
    pub mod unofficial5 {
        pub const ABSOLUTE_X_INDEXED: u8 = 0xFC;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0xF4;
    }
}
pub mod inc {
    pub const ABSOLUTE: u8 = 0xEE;
    pub const ABSOLUTE_X_INDEXED: u8 = 0xFE;
    pub const ZERO_PAGE: u8 = 0xE6;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0xF6;
}
pub mod inx {
    pub const IMPLIED: u8 = 0xE8;
}
pub mod iny {
    pub const IMPLIED: u8 = 0xC8;
}
pub mod isc {
    pub mod unofficial0 {
        pub const X_INDEXED_INDIRECT: u8 = 0xE3;
        pub const ZERO_PAGE: u8 = 0xE7;
        pub const ABSOLUTE: u8 = 0xEF;
        pub const INDIRECT_Y_INDEXED: u8 = 0xF3;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0xF7;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0xFB;
        pub const ABSOLUTE_X_INDEXED: u8 = 0xFF;
    }
}
pub mod jmp {
    pub const ABSOLUTE: u8 = 0x4C;
    pub const INDIRECT: u8 = 0x6C;
}
pub mod jsr {
    pub const ABSOLUTE: u8 = 0x20;
}
pub mod lax {
    pub mod unofficial0 {
        pub const ABSOLUTE: u8 = 0xAF;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0xBF;
        pub const IMMEDIATE: u8 = 0xAB;
        pub const X_INDEXED_INDIRECT: u8 = 0xA3;
        pub const INDIRECT_Y_INDEXED: u8 = 0xB3;
        pub const ZERO_PAGE: u8 = 0xA7;
        pub const ZERO_PAGE_Y_INDEXED: u8 = 0xB7;
    }
}
pub mod lda {
    pub const ABSOLUTE: u8 = 0xAD;
    pub const ABSOLUTE_X_INDEXED: u8 = 0xBD;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0xB9;
    pub const IMMEDIATE: u8 = 0xA9;
    pub const INDIRECT_Y_INDEXED: u8 = 0xB1;
    pub const X_INDEXED_INDIRECT: u8 = 0xA1;
    pub const ZERO_PAGE: u8 = 0xA5;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0xB5;
}
pub mod ldx {
    pub const ABSOLUTE: u8 = 0xAE;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0xBE;
    pub const IMMEDIATE: u8 = 0xA2;
    pub const ZERO_PAGE: u8 = 0xA6;
    pub const ZERO_PAGE_Y_INDEXED: u8 = 0xB6;
}
pub mod ldy {
    pub const ABSOLUTE: u8 = 0xAC;
    pub const ABSOLUTE_X_INDEXED: u8 = 0xBC;
    pub const IMMEDIATE: u8 = 0xA0;
    pub const ZERO_PAGE: u8 = 0xA4;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0xB4;
}
pub mod lsr {
    pub const ABSOLUTE: u8 = 0x4E;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x5E;
    pub const ACCUMULATOR: u8 = 0x4A;
    pub const ZERO_PAGE: u8 = 0x46;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x56;
}
pub mod nop {
    pub const IMPLIED: u8 = 0xEA;
    pub mod unofficial0 {
        pub const IMPLIED: u8 = 0x1A;
    }
    pub mod unofficial1 {
        pub const IMPLIED: u8 = 0x3A;
    }
    pub mod unofficial2 {
        pub const IMPLIED: u8 = 0x5A;
    }
    pub mod unofficial3 {
        pub const IMPLIED: u8 = 0x7A;
    }
    pub mod unofficial4 {
        pub const IMPLIED: u8 = 0xDA;
    }
    pub mod unofficial5 {
        pub const IMPLIED: u8 = 0xFA;
    }
}
pub mod ora {
    pub const ABSOLUTE: u8 = 0x0D;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x1D;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0x19;
    pub const IMMEDIATE: u8 = 0x09;
    pub const INDIRECT_Y_INDEXED: u8 = 0x11;
    pub const X_INDEXED_INDIRECT: u8 = 0x01;
    pub const ZERO_PAGE: u8 = 0x05;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x15;
}
pub mod pha {
    pub const IMPLIED: u8 = 0x48;
}
pub mod php {
    pub const IMPLIED: u8 = 0x08;
}
pub mod pla {
    pub const IMPLIED: u8 = 0x68;
}
pub mod plp {
    pub const IMPLIED: u8 = 0x28;
}
pub mod rla {
    pub mod unofficial0 {
        pub const X_INDEXED_INDIRECT: u8 = 0x23;
        pub const ZERO_PAGE: u8 = 0x27;
        pub const ABSOLUTE: u8 = 0x2F;
        pub const INDIRECT_Y_INDEXED: u8 = 0x33;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x37;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0x3B;
        pub const ABSOLUTE_X_INDEXED: u8 = 0x3F;
    }
}
pub mod rol {
    pub const ABSOLUTE: u8 = 0x2E;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x3E;
    pub const ACCUMULATOR: u8 = 0x2A;
    pub const ZERO_PAGE: u8 = 0x26;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x36;
}
pub mod ror {
    pub const ABSOLUTE: u8 = 0x6E;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x7E;
    pub const ACCUMULATOR: u8 = 0x6A;
    pub const ZERO_PAGE: u8 = 0x66;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x76;
}
pub mod rra {
    pub mod unofficial0 {
        pub const X_INDEXED_INDIRECT: u8 = 0x63;
        pub const ZERO_PAGE: u8 = 0x67;
        pub const ABSOLUTE: u8 = 0x6F;
        pub const INDIRECT_Y_INDEXED: u8 = 0x73;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x77;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0x7B;
        pub const ABSOLUTE_X_INDEXED: u8 = 0x7F;
    }
}
pub mod rti {
    pub const IMPLIED: u8 = 0x40;
}
pub mod rts {
    pub const IMPLIED: u8 = 0x60;
}
pub mod sax {
    pub mod unofficial0 {
        pub const X_INDEXED_INDIRECT: u8 = 0x83;
        pub const ZERO_PAGE: u8 = 0x87;
        pub const ABSOLUTE: u8 = 0x8F;
        pub const ZERO_PAGE_Y_INDEXED: u8 = 0x97;
    }
}
pub mod sbc {
    pub const ABSOLUTE: u8 = 0xED;
    pub const ABSOLUTE_X_INDEXED: u8 = 0xFD;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0xF9;
    pub const IMMEDIATE: u8 = 0xE9;
    pub const INDIRECT_Y_INDEXED: u8 = 0xF1;
    pub const X_INDEXED_INDIRECT: u8 = 0xE1;
    pub const ZERO_PAGE: u8 = 0xE5;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0xF5;
    pub mod unofficial0 {
        pub const IMMEDIATE: u8 = 0xEB;
    }
}
pub mod sec {
    pub const IMPLIED: u8 = 0x38;
}
pub mod sed {
    pub const IMPLIED: u8 = 0xF8;
}
pub mod sei {
    pub const IMPLIED: u8 = 0x78;
}
pub mod skb {
    pub mod unofficial0 {
        pub const IMMEDIATE: u8 = 0x80;
    }
    pub mod unofficial1 {
        pub const IMMEDIATE: u8 = 0x82;
    }
    pub mod unofficial2 {
        pub const IMMEDIATE: u8 = 0x89;
    }
    pub mod unofficial3 {
        pub const IMMEDIATE: u8 = 0xC2;
    }
    pub mod unofficial4 {
        pub const IMMEDIATE: u8 = 0xE2;
    }
}
pub mod slo {
    pub mod unofficial0 {
        pub const X_INDEXED_INDIRECT: u8 = 0x03;
        pub const ZERO_PAGE: u8 = 0x07;
        pub const ABSOLUTE: u8 = 0x0F;
        pub const INDIRECT_Y_INDEXED: u8 = 0x13;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x17;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0x1B;
        pub const ABSOLUTE_X_INDEXED: u8 = 0x1F;
    }
}
pub mod sre {
    pub mod unofficial0 {
        pub const X_INDEXED_INDIRECT: u8 = 0x43;
        pub const ZERO_PAGE: u8 = 0x47;
        pub const ABSOLUTE: u8 = 0x4F;
        pub const INDIRECT_Y_INDEXED: u8 = 0x53;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x57;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0x5B;
        pub const ABSOLUTE_X_INDEXED: u8 = 0x5F;
    }
}
pub mod sta {
    pub const ABSOLUTE: u8 = 0x8D;
    pub const ABSOLUTE_X_INDEXED: u8 = 0x9D;
    pub const ABSOLUTE_Y_INDEXED: u8 = 0x99;
    pub const INDIRECT_Y_INDEXED: u8 = 0x91;
    pub const X_INDEXED_INDIRECT: u8 = 0x81;
    pub const ZERO_PAGE: u8 = 0x85;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x95;
}
pub mod stx {
    pub const ABSOLUTE: u8 = 0x8E;
    pub const ZERO_PAGE: u8 = 0x86;
    pub const ZERO_PAGE_Y_INDEXED: u8 = 0x96;
}
pub mod sty {
    pub const ABSOLUTE: u8 = 0x8C;
    pub const ZERO_PAGE: u8 = 0x84;
    pub const ZERO_PAGE_X_INDEXED: u8 = 0x94;
}
pub mod sxa {
    pub mod unofficial0 {
        pub const ABSOLUTE_Y_INDEXED: u8 = 0x9E;
    }
}
pub mod sya {
    pub mod unofficial0 {
        pub const ABSOLUTE_X_INDEXED: u8 = 0x9C;
    }
}
pub mod tax {
    pub const IMPLIED: u8 = 0xAA;
}
pub mod tay {
    pub const IMPLIED: u8 = 0xA8;
}
pub mod tsx {
    pub const IMPLIED: u8 = 0xBA;
}
pub mod txa {
    pub const IMPLIED: u8 = 0x8A;
}
pub mod txs {
    pub const IMPLIED: u8 = 0x9A;
}
pub mod tya {
    pub const IMPLIED: u8 = 0x98;
}
