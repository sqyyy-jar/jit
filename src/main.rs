use arch::a64::{asm::Asm, reg::Reg, routine::Routine};
use assembler::Assembler;
use std::{
    env::args,
    fs::File,
    io::{Result, Write},
    panic,
};

pub mod arch;
pub mod assembler;
pub mod mem;

fn main() -> Result<()> {
    panic::set_hook(Box::new(|it| {
        println!("Panic: {}", it.to_string());
        std::process::exit(-1);
    }));

    const V: bool = true;
    let mut asm = Asm::default();

    let g_test = asm.const_64(test as usize as _);

    let mut main = Routine::new("main".to_string());
    main.sub_imm12(Reg::X31, Reg::X31, 16);
    main.str_uimm12_offset(Reg::X31, Reg::X30, 0);
    main.br_link("test".to_string());
    main.ldr_global_const(Reg::X9, g_test);
    main.br_reg_link(Reg::X9);
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
    panic!("test print")
}
