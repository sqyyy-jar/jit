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

    let g_test = asm.const_64(test as usize as _);

    let mut main = Routine::new("main".to_string());
    //main.str_imm9_pre_offset(Reg::X30, Reg::X31, -16);
    main.stp_imm7_pre_offset(Reg::X29, Reg::X30, Reg::X31, -2);
    main.mov_sp_to(Reg::X29);
    //main.br_link("test".to_string());
    main.ldr_global_const(Reg::X9, g_test);
    main.br_reg_link(Reg::X9);
    //main.ldr_uimm9_post_offset(Reg::X30, Reg::X31, 16);
    main.ldp_imm7_post_offset(Reg::X29, Reg::X30, Reg::X31, 2);
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

#[no_mangle]
pub extern "C" fn test() {
    panic!();
}
