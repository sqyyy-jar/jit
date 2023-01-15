use crate::{
    assembler::{Assembler, PostOp, Subroutine, VTable},
    mem::{self, MemoryView, RawMemoryView, VecMemoryView},
};
use std::{collections::HashMap, mem::transmute};

pub struct Asm {
    finalizing: bool,
    routines: Vec<Routine>,
    vtable: HashMap<String, usize>,
}

impl Asm {
    fn int_jit<'a>(&mut self, view: &mut impl MemoryView<'a>) {
        let mut offset = 0;
        let mut post_op_map = Vec::new();
        let address = view.address();
        while let Some(Routine {
            name,
            code,
            post_ops,
        }) = self.routines.pop()
        {
            post_op_map.push((offset, code.len(), post_ops));
            self.vtable.insert(name, address + offset);
            for byte in code {
                view.push(byte);
                offset += 1;
            }
        }
        for (offset, len, post_ops) in post_op_map {
            for post_op in post_ops {
                post_op.process(self, address + offset, view.slice_at_mut(offset, len));
            }
        }
    }
}

impl Assembler for Asm {
    type AsmRoutine = Routine;

    fn get_label_address(&self, name: &str) -> usize {
        if !self.finalizing {
            return usize::MAX;
        }
        self.vtable.get(name).cloned().unwrap_or(0)
    }

    fn jit(mut self) -> Option<VTable> {
        self.finalizing = true;
        let size: usize = self.routines.iter().map(|it| it.code.len()).sum();
        let ptr = mem::alloc_aligned(size);
        if ptr.is_null() {
            dbg!("Could not allocate memory");
            return None;
        }
        self.int_jit(&mut RawMemoryView::new(ptr));
        if !mem::make_executable_aligned(ptr, size) {
            unsafe {
                mem::dealloc_aligned(ptr, size);
            }
            dbg!("Could not make memory executable");
            return None;
        }
        Some(VTable::new(
            ptr,
            size,
            self.vtable
                .into_iter()
                .map(|(k, v)| (k, unsafe { transmute(v) }))
                .collect(),
        ))
    }

    fn virtual_jit(mut self) -> Option<(Vec<u8>, HashMap<String, usize>)> {
        self.finalizing = true;
        let size: usize = self.routines.iter().map(|it| it.code.len()).sum();
        let mut vec = Vec::with_capacity(size);
        let mut view = VecMemoryView::new(0, &mut vec);
        self.int_jit(&mut view);
        let address = view.address();
        Some((
            vec,
            self.vtable
                .into_iter()
                .map(|(k, v)| (k, v - address))
                .collect(),
        ))
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
                if addr == usize::MAX {
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
                if addr == usize::MAX {
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
