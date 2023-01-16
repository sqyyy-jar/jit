use std::{alloc, slice};

pub const fn align(size: usize, align: usize) -> usize {
    if size % align == 0 {
        size
    } else {
        size - (size % align) + align
    }
}

pub fn get_system_alignment() -> usize {
    #[cfg(unix)]
    {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }
    #[cfg(windows)]
    {
        use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
        let mut info = SYSTEM_INFO::default();
        unsafe {
            GetSystemInfo(&mut info);
        }
        info.dwPageSize as usize
    }
}

pub fn alloc_aligned(size: usize) -> *mut u8 {
    unsafe {
        let alignment = get_system_alignment();
        let size = align(size, alignment);
        alloc::alloc_zeroed(alloc::Layout::from_size_align_unchecked(
            align(size, alignment),
            alignment,
        ))
    }
}

/// # Safety
/// This function will deallocate the given pointer without any safety checks.
pub unsafe fn dealloc_aligned(ptr: *mut u8, size: usize) {
    let alignment = get_system_alignment();
    let size = align(size, alignment);
    alloc::dealloc(
        ptr,
        alloc::Layout::from_size_align_unchecked(align(size, alignment), alignment),
    );
}

pub fn make_executable_aligned(ptr: *mut u8, size: usize) -> bool {
    unsafe {
        let alignment = get_system_alignment();
        #[cfg(unix)]
        {
            libc::mprotect(
                ptr as *mut _,
                align(size, alignment),
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            ) == 0
        }
        #[cfg(windows)]
        {
            use windows::Win32::System::Memory;
            let mut _old_fp = Memory::PAGE_PROTECTION_FLAGS::default();
            Memory::VirtualProtect(
                ptr as *mut _,
                align(size, alignment),
                Memory::PAGE_EXECUTE_READ,
                &mut _old_fp,
            )
            .as_bool()
        }
    }
}

pub fn make_readwrite_aligned(ptr: *mut u8, size: usize) -> bool {
    unsafe {
        let alignment = get_system_alignment();
        #[cfg(unix)]
        {
            libc::mprotect(
                ptr as *mut _,
                align(size, alignment),
                libc::PROT_READ | libc::PROT_WRITE,
            ) == 0
        }
        #[cfg(windows)]
        {
            use windows::Win32::System::Memory;
            let mut _old_fp = Memory::PAGE_PROTECTION_FLAGS::default();
            Memory::VirtualProtect(
                ptr as *mut _,
                align(size, alignment),
                Memory::PAGE_READWRITE,
                &mut _old_fp,
            )
            .as_bool()
        }
    }
}

pub trait MemoryView<'a> {
    fn address(&self) -> usize;

    fn push(&mut self, byte: u8);

    fn slice_at(&self, from: usize, size: usize) -> &[u8];

    fn slice_at_mut(&mut self, from: usize, size: usize) -> &mut [u8];
}

pub struct RawMemoryView {
    ptr: *mut u8,
    index: usize,
}

impl RawMemoryView {
    pub fn new(ptr: *mut u8) -> Self {
        Self { ptr, index: 0 }
    }
}

impl<'a> MemoryView<'a> for RawMemoryView {
    fn address(&self) -> usize {
        self.ptr as usize
    }

    fn push(&mut self, byte: u8) {
        unsafe { *self.ptr.add(self.index) = byte };
        self.index += 1;
    }

    fn slice_at(&self, from: usize, size: usize) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.add(from), size) }
    }

    fn slice_at_mut(&mut self, from: usize, size: usize) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.add(from), size) }
    }
}

pub struct VecMemoryView<'a> {
    address: usize,
    vec: &'a mut Vec<u8>,
}

impl<'a> VecMemoryView<'a> {
    pub fn new(address: usize, vec: &'a mut Vec<u8>) -> Self {
        Self { address, vec }
    }
}

impl<'a> MemoryView<'a> for VecMemoryView<'a> {
    fn address(&self) -> usize {
        self.address
    }

    fn push(&mut self, byte: u8) {
        self.vec.push(byte);
    }

    fn slice_at(&self, from: usize, size: usize) -> &[u8] {
        &self.vec[from..from + size]
    }

    fn slice_at_mut(&mut self, from: usize, size: usize) -> &mut [u8] {
        &mut self.vec[from..from + size]
    }
}
