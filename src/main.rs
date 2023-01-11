use std::slice;

pub mod assembler;
pub mod concept;
pub mod mem;
pub mod arch;

fn main() {
    let mut bytes = [0u8; 16];
    let r = bytes.as_mut_ptr();
    let x = unsafe { slice::from_raw_parts_mut(r, 16) };
    x[0] = 255;
    println!("{bytes:?}");
}
