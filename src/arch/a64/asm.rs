use super::routine::Routine;
use crate::{
    assembler::{Assembler, PostOp, Subroutine, VTable},
    mem::{self, MemoryView, RawMemoryView, VecMemoryView},
};
use std::{collections::HashMap, mem::transmute};

pub struct Asm {
    finalizing: bool,
    constants: Vec<u8>,
    routines: Vec<Routine>,
    vtable: HashMap<String, usize>,
    const_addr: usize,
}

impl Asm {
    fn int_jit<'a>(&mut self, view: &mut impl MemoryView<'a>) {
        self.finalizing = true;
        let address = view.address();
        self.const_addr = address;
        let mut offset = 0;
        let mut post_op_map = Vec::new();
        for byte in &self.constants {
            view.push(*byte);
            offset += 1;
        }
        while let Some(Routine {
            name,
            constants,
            code,
            post_ops,
        }) = self.routines.pop()
        {
            post_op_map.push((offset, constants.len(), code.len(), post_ops));
            for byte in constants {
                view.push(byte);
                offset += 1;
            }
            self.vtable.insert(name, address + offset);
            for byte in code {
                view.push(byte);
                offset += 1;
            }
        }
        for (offset, const_len, code_len, post_ops) in post_op_map {
            for post_op in post_ops {
                post_op.process(
                    self,
                    address + offset,
                    const_len,
                    view.slice_at_mut(offset, const_len + code_len),
                );
            }
        }
    }

    pub fn const_32(&mut self, value: u32) -> usize {
        let index = self.constants.len() / 4;
        for byte in value.to_ne_bytes() {
            self.constants.push(byte);
        }
        index
    }

    pub fn const_64(&mut self, value: u64) -> usize {
        let index = self.constants.len() / 4;
        for byte in value.to_ne_bytes() {
            self.constants.push(byte);
        }
        index
    }

    pub fn push_routine(&mut self, routine: Routine) {
        self.routines.push(routine);
    }
}

impl Default for Asm {
    fn default() -> Self {
        Self {
            finalizing: false,
            constants: Vec::with_capacity(0),
            routines: Vec::with_capacity(0),
            vtable: HashMap::with_capacity(0),
            const_addr: 0,
        }
    }
}

impl Assembler for Asm {
    type AsmRoutine = Routine;

    fn global_const_address(&self) -> usize {
        if !self.finalizing {
            panic!("Illegal access");
        }
        self.const_addr
    }

    fn get_label_address(&self, name: &str) -> Option<usize> {
        if !self.finalizing {
            panic!("Illegal access");
        }
        self.vtable.get(name).cloned()
    }

    fn jit(mut self) -> Option<VTable> {
        let size = self.constants.len()
            + self
                .routines
                .iter()
                .map(|it| it.constants().len() + it.code().len())
                .sum::<usize>();
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
        let size: usize = self.routines.iter().map(|it| it.code().len()).sum();
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
