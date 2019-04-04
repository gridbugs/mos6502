use crate::*;
use test_framework::test_sample;

#[test]
fn arithmetic() {
    test_sample(Arithmetic);
}

#[test]
fn infinite_loop() {
    test_sample(InfiniteLoop);
}

#[test]
fn jump_indirect() {
    test_sample(JumpIndirect);
}

#[test]
fn load_accumulator_immediate() {
    test_sample(LoadAccumulatorImmediate);
}

#[test]
fn load_and_store_all_addressing_modes() {
    test_sample(LoadAndStoreAllAddressingModes);
}

#[test]
fn stack_basic() {
    test_sample(StackBasic);
}

#[test]
fn stack_status_register() {
    test_sample(StackStatusRegister);
}

#[test]
fn store_accumulator() {
    test_sample(StoreAccumulator);
}
