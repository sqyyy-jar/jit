use std::alloc;

pub fn align(size: usize, align: usize) -> usize {
    if size % align == 0 {
        size
    } else {
        size - (size % align) + align
    }
}

pub fn get_system_alignment() -> usize {
    #[cfg(unix)]
    {
        libc::sysconf(libc::_SC_PAGESIZE) as usize
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
        let size = align(size, 0);
        alloc::alloc_zeroed(alloc::Layout::from_size_align_unchecked(
            align(size, alignment),
            alignment,
        ))
    }
}

pub fn make_executable(ptr: *mut u8, size: usize) -> bool {
    unsafe {
        let alignment = get_system_alignment();
        //#[cfg(unix)]
        {
            libc::mprotect()
        }
        #[cfg(windows)]
        {
            todo!()
        }
    }
}
