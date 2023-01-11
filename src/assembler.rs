pub trait Assembler {}

pub trait Subroutine {
    fn process(&self, assembler: &impl Assembler, address: usize, bytes: &mut [u8]);
}

pub trait PostOp {
    fn process(&self, assembler: &impl Assembler, address: usize, bytes: &mut [u8]);
}
