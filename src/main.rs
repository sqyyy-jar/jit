use arch::a64::{asm::Asm, reg::Reg, routine::Routine};
use assembler::Assembler;
use std::{
    env::args,
    fs::File,
    io::{Result, Write},
};

pub mod arch;
pub mod assembler;
pub mod mem;

fn main() -> Result<()> {
    let mut f = File::create(args().nth(1).unwrap())?;
    let mut asm = Asm::default();

    let mut main = Routine::new("main".to_string());
    main.mov_imm16(Reg::X10, 0xCAFE);
    main.br_link("test".to_string());
    main.ret();
    asm.push_routine(main);

    let mut test = Routine::new("test".to_string());
    test.mov_imm16(Reg::X30, 0xDEAD);
    test.br("main".to_string());
    test.ret();
    asm.push_routine(test);

    let (code, vtable) = asm.virtual_jit().unwrap();
    f.write_all(&code)?;
    println!("VTable: {vtable:#?}");
    Ok(())
}
