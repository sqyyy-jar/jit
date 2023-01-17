use crate::{
    arch::a64::reg::{is_64_bit, Reg},
    assembler::Assembler,
};

pub fn br_reg(bytes: &mut [u8], insn_offset: usize, dst_reg: Reg) {
    assert!(is_64_bit(dst_reg), "Branch register must be 64-bit");
    write_ne_32(
        bytes,
        insn_offset,
        0xD61F0000 | ((dst_reg as u32 & 0x1F) << 5),
    );
}

pub fn br_reg_link(bytes: &mut [u8], insn_offset: usize, dst_reg: Reg) {
    assert!(is_64_bit(dst_reg), "Branch register must be 64-bit");
    write_ne_32(
        bytes,
        insn_offset,
        0xD63F0000 | 0xD61F0000 | ((dst_reg as u32 & 0x1F) << 5),
    );
}

pub fn load_const(bytes: &mut [u8], insn_offset: usize, dst_reg: Reg, const_offset: usize) {
    let rel = const_offset as isize - (insn_offset as isize / 4);
    assert!(
        (-0x40000..=0x3FFFF).contains(&rel),
        "Tried to load constant that is not in range"
    );
    write_ne_32(
        bytes,
        insn_offset,
        0x18000000
            | ((is_64_bit(dst_reg) as u32) << 30)
            | ((rel as u32 & 0x7FFFF) << 5)
            | (dst_reg as u32 & 0x1F),
    );
}

pub fn load_global_const(
    asm: &impl Assembler,
    abs_addr: usize,
    bytes: &mut [u8],
    insn_offset: usize,
    dst_reg: Reg,
    const_offset: usize,
) {
    let rel = (asm.global_const_address() as isize + const_offset as isize
        - abs_addr as isize
        - insn_offset as isize)
        / 4;
    assert!(
        (-0x40000..=0x3FFFF).contains(&rel),
        "Tried to load constant that is not in range"
    );
    write_ne_32(
        bytes,
        insn_offset,
        0x18000000
            | ((is_64_bit(dst_reg) as u32) << 30)
            | ((rel as u32 & 0x7FFFF) << 5)
            | (dst_reg as u32 & 0x1F),
    );
}

pub fn ldr_imm9_post_offset(dst_reg: Reg, src_reg: Reg, imm9: i16) -> u32 {
    assert!(is_64_bit(dst_reg), "Destination register must be 64-bit");
    0xB8400400
        | ((is_64_bit(src_reg) as u32) << 30)
        | ((imm9 as u32 & 0x1FF) << 12)
        | ((dst_reg as u32 & 0x1F) << 5)
        | (src_reg as u32 & 0x1F)
}

pub fn str_imm9_pre_offset(dst_reg: Reg, src_reg: Reg, imm9: i16) -> u32 {
    assert!(is_64_bit(dst_reg), "Destination register must be 64-bit");
    0xB8000C00
        | ((is_64_bit(src_reg) as u32) << 30)
        | ((imm9 as u32 & 0x1FF) << 12)
        | ((dst_reg as u32 & 0x1F) << 5)
        | (src_reg as u32 & 0x1F)
}

pub fn write_ne_32(slice: &mut [u8], index: usize, value: u32) {
    for (offset, byte) in value.to_ne_bytes().into_iter().enumerate() {
        slice[index + offset] = byte;
    }
}
