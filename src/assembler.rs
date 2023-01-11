use std::collections::HashMap;

pub trait Assembler {
    type AsmRoutine: Subroutine;

    fn get_label_address(&self, name: &str) -> usize;

    fn jit(self) -> HashMap<String, fn()>;
}

pub trait Subroutine {
    fn bytes(&self) -> &[u8];

    fn process(&self, assembler: &impl Assembler, abs_addr: usize, bytes: &mut [u8]);
}

pub trait PostOp {
    fn process(&self, assembler: &impl Assembler, abs_addrs: usize, bytes: &mut [u8]);
}
