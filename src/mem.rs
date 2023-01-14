use std::alloc;

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

pub fn make_executable(ptr: *mut u8, size: usize) -> bool {
    unsafe {
        let alignment = get_system_alignment();
        #[cfg(unix)]
        {
            libc::mprotect(
                ptr as *mut _,
                align(size, alignment),
                libc::PROT_READ | libc::PROT_EXEC,
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

pub fn make_readwrite(ptr: *mut u8, size: usize) -> bool {
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
