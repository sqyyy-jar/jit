use crate::{
    assembler::{Assembler, PostOp, Subroutine, VTable},
    mem,
};
use std::{collections::HashMap, mem::transmute, slice};

pub struct Asm {
    finalizing: bool,
    routines: Vec<Routine>,
    vtable: HashMap<String, fn()>,
}

impl Assembler for Asm {
    type AsmRoutine = Routine;

    fn get_label_address(&self, name: &str) -> usize {
        if !self.finalizing {
            return 0;
        }
        self.vtable.get(name).map_or_else(|| 0, |it| *it as usize)
    }

    fn jit(mut self) -> Option<VTable> {
        self.finalizing = true;
        let size: usize = self.routines.iter().map(|it| it.code.len()).sum();
        let ptr = mem::alloc_aligned(size);
        if ptr.is_null() {
            dbg!("Could not allocate memory");
            return None;
        }
        let mut offset = 0;
        let mut post_op_map = Vec::new();
        while let Some(Routine {
            name,
            code,
            post_ops,
        }) = self.routines.pop()
        {
            post_op_map.push((offset, code.len(), post_ops));
            self.vtable
                .insert(name, unsafe { transmute(ptr.add(offset)) });
            for byte in code {
                unsafe {
                    *ptr.add(offset) = byte;
                }
                offset += 1;
            }
        }
        for (offset, len, post_ops) in post_op_map {
            for post_op in post_ops {
                post_op.process(&self, unsafe { ptr.add(offset) } as _, unsafe {
                    slice::from_raw_parts_mut(ptr.add(offset), len)
                });
            }
        }
        if !mem::make_executable_aligned(ptr, size) {
            unsafe {
                mem::dealloc_aligned(ptr, size);
            }
            dbg!("Could not make memory executable");
            return None;
        }
        Some(VTable::new(ptr, size, self.vtable))
    }
}

pub struct Routine {
    name: String,
    code: Vec<u8>,
    post_ops: Vec<Op>,
}

impl Subroutine for Routine {
    fn bytes(&self) -> &[u8] {
        &self.code
    }

    fn process(&self, assembler: &impl Assembler, abs_addr: usize, bytes: &mut [u8]) {
        for op in &self.post_ops {
            op.process(assembler, abs_addr, bytes);
        }
    }
}

pub enum Op {
    Branch { offset: usize, label: String },
    BranchWithLink { offset: usize, label: String },
}

impl PostOp for Op {
    fn process(&self, assembler: &impl Assembler, abs_addr: usize, bytes: &mut [u8]) {
        match self {
            Self::Branch { offset, label } => {
                let addr = assembler.get_label_address(label);
                if addr == 0 {
                    panic!("Tried to branch to non-existent label");
                }
                let rel = addr as isize - abs_addr as isize;
                if !(-0x2000000..=0x1FFFFFF).contains(&rel) {
                    panic!("Tried to branch to label not in range");
                }
                let insn = (0x14000000u32 | (rel as u32 & 0x3FFFFFF)).to_le_bytes();
                for (idx, byte) in insn.iter().enumerate() {
                    bytes[offset * 4 + idx] = *byte;
                }
            }
            Self::BranchWithLink { offset, label } => {
                let addr = assembler.get_label_address(label);
                if addr == 0 {
                    panic!("Tried to branch to non-existent label");
                }
                let rel = addr as isize - abs_addr as isize;
                if !(-0x2000000..=0x1FFFFFF).contains(&rel) {
                    panic!("Tried to branch to label not in range");
                }
                let insn = (0x94000000u32 | (rel as u32 & 0x3FFFFFF)).to_le_bytes();
                for (idx, byte) in insn.iter().enumerate() {
                    bytes[offset * 4 + idx] = *byte;
                }
            }
        }
    }
}
