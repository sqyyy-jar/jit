use crate::assembler::{Assembler, PostOp, Subroutine};
use std::collections::HashMap;

pub struct Asm {
    finalizing: bool,
    routines: Vec<Routine>,
    labels: HashMap<String, fn()>,
}

impl Assembler for Asm {
    type AsmRoutine = Routine;

    fn get_label_address(&self, name: &str) -> usize {
        if !self.finalizing {
            return 0;
        }
        self.labels.get(name).map_or_else(|| 0, |it| *it as usize)
    }

    fn jit(self) -> HashMap<String, fn()> {
        let size: usize = self.routines.iter().map(|it| it.code.len()).sum();
        
        todo!()
    }
}

pub struct Routine {
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
