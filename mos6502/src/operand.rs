pub trait Trait {
    fn instruction_bytes() -> u16;
}

pub struct None;
impl Trait for None {
    fn instruction_bytes() -> u16 {
        1
    }
}

pub struct Byte;
impl Trait for Byte {
    fn instruction_bytes() -> u16 {
        2
    }
}

pub struct Address;
impl Trait for Address {
    fn instruction_bytes() -> u16 {
        3
    }
}
