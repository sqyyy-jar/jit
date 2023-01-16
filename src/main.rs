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
    const V: bool = true;
    let mut asm = Asm::default();

    let mut main = Routine::new("main".to_string());
    main.sub_imm12(Reg::X31, Reg::X31, 16);
    main.str_uimm12_offset(Reg::X31, Reg::X30, 0);
    main.br_link("test".to_string());
    main.br_extern_link(test as usize);
    main.ldr_uimm12_offset(Reg::X30, Reg::X31, 0);
    main.add_imm12(Reg::X31, Reg::X31, 16);
    main.ret();
    asm.push_routine(main);

    let mut test = Routine::new("test".to_string());
    test.ret();
    asm.push_routine(test);

    if V {
        let mut f = File::create(args().nth(1).unwrap())?;
        let (code, vtable) = asm.virtual_jit().unwrap();
        f.write_all(&code)?;
        println!("VTable: {vtable:#?}");
    } else {
        let vtable = asm.jit().unwrap();
        vtable.lookup("main").unwrap()();
    }
    Ok(())
}

extern "C" fn test() {
    panic!("test panic")
}
