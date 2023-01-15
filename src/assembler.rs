use crate::mem;
use std::collections::HashMap;

pub trait Assembler {
    type AsmRoutine: Subroutine;

    fn get_label_address(&self, label: &str) -> usize;

    fn jit(self) -> Option<VTable>;
}

pub trait Subroutine {
    fn bytes(&self) -> &[u8];

    fn process(&self, assembler: &impl Assembler, abs_addr: usize, bytes: &mut [u8]);
}

pub trait PostOp {
    fn process(&self, assembler: &impl Assembler, abs_addrs: usize, bytes: &mut [u8]);
}

pub struct VTable {
    ptr: *mut u8,
    size: usize,
    table: HashMap<String, fn()>,
}

impl VTable {
    pub fn new(ptr: *mut u8, size: usize, table: HashMap<String, fn()>) -> Self {
        Self { ptr, size, table }
    }

    pub fn lookup(&self, label: &str) -> Option<fn()> {
        self.table.get(label).cloned()
    }
}

impl Drop for VTable {
    fn drop(&mut self) {
        if !mem::make_readwrite_aligned(self.ptr, self.size) {
            panic!("Could not drop VTable")
        }
        unsafe {
            mem::dealloc_aligned(self.ptr, self.size);
        }
    }
}
