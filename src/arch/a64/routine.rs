use super::{
    raw::{self, write_ne_32},
    reg::{is_64_bit, Reg},
};
use crate::assembler::{Assembler, PostOp, Subroutine};

pub struct Routine {
    pub(super) name: String,
    pub(super) constants: Vec<u8>,
    pub(super) code: Vec<u8>,
    pub(super) post_ops: Vec<Op>,
}

impl Routine {
    pub fn new(name: String) -> Self {
        Self {
            name,
            constants: Vec::with_capacity(0),
            code: Vec::with_capacity(0),
            post_ops: Vec::with_capacity(0),
        }
    }

    /// Returns from a routine
    pub fn ret(&mut self) {
        self.int_insn(0xD65F03C0);
    }

    /// Placeholder
    pub fn nop(&mut self) {
        self.int_insn(0xD503201F);
    }

    /// Moves the 16-bit integer into the specified register
    pub fn mov_imm16(&mut self, dst_reg: Reg, imm: u16) {
        self.int_insn(
            0x52800000
                | ((is_64_bit(dst_reg) as u32) << 31)
                | ((imm as u32) << 5)
                | (dst_reg as u32 & 0x1f),
        );
    }

    /// Moves the the value stored in the source register into the destination register
    pub fn mov_reg(&mut self, dst_reg: Reg, src_reg: Reg) {
        let bits_64 = is_64_bit(dst_reg);
        if bits_64 != is_64_bit(src_reg) {
            panic!("Both registers must be of equal size");
        }
        self.int_insn(
            0x2A0003E0
                | ((bits_64 as u32) << 31)
                | ((src_reg as u32 & 0x1F) << 16)
                | (dst_reg as u32 & 0x1F),
        );
    }

    /// Branches to a 26-bit address relative to the first byte of the instruction inserted through
    /// this
    ///
    /// The relative address has to be multiplied by 4 to get the bytes to be branched
    pub fn br_rel(&mut self, rel26: i32) {
        self.int_insn(0x14000000 | (rel26 as u32 & 0x3FFFFFF));
    }

    /// Branches to the absolute address stored in a register
    pub fn br_reg(&mut self, dst_reg: Reg) {
        assert!(is_64_bit(dst_reg), "Branch register must be 64-bit");
        raw::br_reg(self.alloc_insn(), 0, dst_reg);
    }

    /// Branches to a 26-bit address relative to the first byte of the instruction inserted through
    /// this storing PC+4 in the X30 register
    ///
    /// The relative address has to be multiplied by 4 to get the bytes to be branched
    pub fn br_rel_link(&mut self, rel26: i32) {
        self.int_insn(0x94000000 | (rel26 as u32 & 0x3FFFFFF));
    }

    /// Branches to the absolute address stored in a register storing PC+4 in the X30 register
    pub fn br_reg_link(&mut self, dst_reg: Reg) {
        assert!(is_64_bit(dst_reg), "Branch register must be 64-bit");
        raw::br_reg_link(self.alloc_insn(), 0, dst_reg);
    }

    /// Branches to label that must be present in the V-Table
    pub fn br(&mut self, label: String) {
        self.post_ops.push(Op::Branch {
            insn_offset: self.code.len(),
            label,
        });
        self.nop();
    }

    /// Branches to the absolute address stored in a register storing PC+4 in the X30 register
    pub fn br_link(&mut self, label: String) {
        self.post_ops.push(Op::BranchWithLink {
            insn_offset: self.code.len(),
            label,
        });
        self.nop();
    }

    /// Subtracts the immediate 12-bit value from the value stored in lhs and puts the result
    /// into the destination register
    pub fn sub_imm12(&mut self, dst_reg: Reg, lhs: Reg, imm12: u16) {
        let bits_64 = is_64_bit(dst_reg);
        if bits_64 != is_64_bit(lhs) {
            panic!("Both registers must be of equal size");
        }
        self.int_insn(
            0x51000000
                | ((bits_64 as u32) << 31)
                | ((imm12 as u32 & 0xFFF) << 10)
                | ((lhs as u32 & 0x1F) << 5)
                | (dst_reg as u32 & 0x1F),
        );
    }

    /// Adds the immediate 12-bit value from the value stored in lhs and puts the result into the
    /// destination register
    pub fn add_imm12(&mut self, dst_reg: Reg, lhs: Reg, imm12: u16) {
        let bits_64 = is_64_bit(dst_reg);
        if bits_64 != is_64_bit(lhs) {
            panic!("Both registers must be of equal size");
        }
        self.int_insn(
            0x11000000
                | ((bits_64 as u32) << 31)
                | ((imm12 as u32 & 0xFFF) << 10)
                | ((lhs as u32 & 0x1F) << 5)
                | (dst_reg as u32 & 0x1F),
        );
    }

    /// Stores the value of `src_reg` into the address `dst_reg + imm12` where `imm12` is unsigned
    ///
    /// If `src_reg` is 32-bit `imm12` will be multiplied by 4 before adding it to `dst_reg`
    ///
    /// If `src_reg` is 64-bit `imm12` will be multiplied by 8 before adding it to `dst_reg`
    pub fn str_uimm12_offset(&mut self, dst_reg: Reg, src_reg: Reg, imm12: u16) {
        if !is_64_bit(dst_reg) {
            panic!("Destination register must be 64-bit");
        }
        self.int_insn(
            0xB9000000
                | ((is_64_bit(src_reg) as u32) << 30)
                | ((imm12 as u32 & 0xFFF) << 10)
                | ((dst_reg as u32 & 0x1F) << 5)
                | (src_reg as u32 & 0x1F),
        );
    }

    /// Stores the value of `src_reg` into the address `dst_reg + imm9` where `imm9` is signed
    /// and adds `imm9` to `dst_reg` afterwards
    ///
    /// If `src_reg` is 32-bit `imm9` will be multiplied by 4 before adding it to `dst_reg`
    ///
    /// If `src_reg` is 64-bit `imm9` will be multiplied by 8 before adding it to `dst_reg`
    pub fn str_imm9_pre_offset(&mut self, dst_reg: Reg, src_reg: Reg, imm9: i16) {
        write_ne_32(
            self.alloc_insn(),
            0,
            super::raw::str_imm9_pre_offset(dst_reg, src_reg, imm9),
        );
    }

    /// Loads the value of address `src_reg + imm12` into `dst_reg` where `imm12` is unsigned
    ///
    /// If `dst_reg` is 32-bit `imm12` will be multiplied by 4 before adding it to `src_reg`
    ///
    /// If `dst_reg` is 64-bit `imm12` will be multiplied by 8 before adding it to `src_reg`
    pub fn ldr_uimm12_offset(&mut self, dst_reg: Reg, src_reg: Reg, imm12: u16) {
        if !is_64_bit(src_reg) {
            panic!("Source register must be 64-bit");
        }
        self.int_insn(
            0xB9400000
                | ((is_64_bit(src_reg) as u32) << 30)
                | ((imm12 as u32 & 0xFFF) << 10)
                | ((src_reg as u32 & 0x1F) << 5)
                | (dst_reg as u32 & 0x1F),
        );
    }

    /// Loads the value of address `src_reg + imm9` into `dst_reg` where `imm9` is signed
    ///
    /// If `dst_reg` is 32-bit `imm9` will be multiplied by 4 before adding it to `src_reg`
    ///
    /// If `dst_reg` is 64-bit `imm9` will be multiplied by 8 before adding it to `src_reg`
    pub fn ldr_uimm9_post_offset(&mut self, dst_reg: Reg, src_reg: Reg, imm9: i16) {
        write_ne_32(
            self.alloc_insn(),
            0,
            super::raw::ldr_imm9_post_offset(dst_reg, src_reg, imm9),
        );
    }

    /// Loads the value of relative address `rel19` into `dst_reg` where `rel19` is signed
    ///
    /// `rel19` will be multiplied by 4 before addressing
    pub fn ldr_rel19(&mut self, dst_reg: Reg, rel19: i32) {
        self.int_insn(
            0x18000000
                | ((is_64_bit(dst_reg) as u32) << 30)
                | ((rel19 as u32 & 0x7FFFF) << 5)
                | (dst_reg as u32 & 0x1F),
        );
    }

    /// Loads the value of a constant
    ///
    /// `offset` is the offset of the constant, represented in 32-bit steps
    pub fn ldr_const(&mut self, dst_reg: Reg, offset: usize) {
        self.post_ops.push(Op::LoadConst {
            insn_offset: self.code.len(),
            dst_reg,
            const_offset: offset,
        });
        self.nop();
    }

    /// Loads the value of a global constant
    ///
    /// `offset` is the offset of the constant, represented in 32-bit steps
    pub fn ldr_global_const(&mut self, dst_reg: Reg, offset: usize) {
        self.post_ops.push(Op::LoadGlobalConst {
            insn_offset: self.code.len(),
            dst_reg,
            const_offset: offset,
        });
        self.nop();
    }

    /// Stores a 32-bit constant in front of the code
    ///
    /// Return the index of the constant
    pub fn const_32(&mut self, value: u32) -> usize {
        let index = self.constants.len() / 4;
        for byte in value.to_ne_bytes() {
            self.constants.push(byte);
        }
        index
    }

    /// Stores a 64-bit constant in front of the code
    ///
    /// Return the index of the constant
    pub fn const_64(&mut self, value: u64) -> usize {
        let index = self.constants.len() / 4;
        for byte in value.to_ne_bytes() {
            self.constants.push(byte);
        }
        index
    }

    fn int_insn(&mut self, value: u32) {
        for byte in value.to_ne_bytes() {
            self.code.push(byte);
        }
    }

    fn alloc_insn(&mut self) -> &mut [u8] {
        let start = self.code.len();
        for _ in 0..4 {
            self.code.push(0);
        }
        let end = self.code.len();
        &mut self.code[start..end]
    }
}

impl Subroutine for Routine {
    fn constants(&self) -> &[u8] {
        &self.constants
    }

    fn code(&self) -> &[u8] {
        &self.code
    }

    fn process(
        &self,
        assembler: &impl Assembler,
        abs_addr: usize,
        code_offset: usize,
        bytes: &mut [u8],
    ) {
        for op in &self.post_ops {
            op.process(assembler, abs_addr, code_offset, bytes);
        }
    }
}

pub enum Op {
    Branch {
        insn_offset: usize,
        label: String,
    },
    BranchWithLink {
        insn_offset: usize,
        label: String,
    },
    LoadConst {
        insn_offset: usize,
        dst_reg: Reg,
        const_offset: usize,
    },
    LoadGlobalConst {
        insn_offset: usize,
        dst_reg: Reg,
        const_offset: usize,
    },
}

impl PostOp for Op {
    fn process(
        &self,
        assembler: &impl Assembler,
        abs_addr: usize,
        code_offset: usize,
        bytes: &mut [u8],
    ) {
        match self {
            Self::Branch { insn_offset, label } => {
                let Some(addr) = assembler.get_label_address(label) else {
                    panic!("Tried to branch to non-existent label");
                };
                let rel = (addr as isize
                    - code_offset as isize
                    - *insn_offset as isize
                    - abs_addr as isize)
                    / 4;
                if !(-0x2000000..=0x1FFFFFF).contains(&rel) {
                    panic!("Tried to branch to label not in range");
                }
                write_ne_32(
                    bytes,
                    code_offset + insn_offset,
                    0x14000000u32 | (rel as u32 & 0x3FFFFFF),
                );
            }
            Self::BranchWithLink { insn_offset, label } => {
                let Some(addr) = assembler.get_label_address(label) else {
                    panic!("Tried to branch to non-existent label");
                };
                let rel = (addr as isize
                    - code_offset as isize
                    - *insn_offset as isize
                    - abs_addr as isize)
                    / 4;
                if !(-0x2000000..=0x1FFFFFF).contains(&rel) {
                    panic!("Tried to branch to label not in range");
                }
                write_ne_32(
                    bytes,
                    code_offset + insn_offset,
                    0x94000000u32 | (rel as u32 & 0x3FFFFFF),
                );
            }
            Self::LoadConst {
                insn_offset,
                dst_reg,
                const_offset,
            } => {
                raw::load_const(bytes, code_offset + *insn_offset, *dst_reg, *const_offset);
            }
            Self::LoadGlobalConst {
                insn_offset,
                dst_reg,
                const_offset,
            } => {
                raw::load_global_const(
                    assembler,
                    abs_addr,
                    bytes,
                    code_offset + *insn_offset,
                    *dst_reg,
                    *const_offset,
                );
            }
        }
    }
}
