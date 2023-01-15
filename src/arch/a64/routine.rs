use crate::assembler::{Assembler, PostOp, Subroutine};

pub struct Routine {
    pub(crate) name: String,
    pub(crate) code: Vec<u8>,
    pub(crate) post_ops: Vec<Op>,
}

impl Routine {
    pub fn new(name: String) -> Self {
        Self {
            name,
            code: Vec::with_capacity(0),
            post_ops: Vec::with_capacity(0),
        }
    }
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
