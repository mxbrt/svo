pub struct Slice<'a, T>(pub &'a mut [T]);

impl<'a, T> Slice<'a, T> {
    pub fn new(len: usize) -> Self {
        let len = len.next_power_of_two() << 1;
        if len == 0 {
            panic!("SPGrid size overflow");
        }

        let data = unsafe {
            let ptr = libc::mmap(
                0 as *mut libc::c_void,
                len * std::mem::size_of::<T>(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            );
            if ptr == libc::MAP_FAILED {
                let err = std::ffi::CStr::from_ptr(libc::strerror(*libc::__errno_location()));
                panic!("SparseSlice: libc::mmap: {}", err.to_str().unwrap());
            }
            std::slice::from_raw_parts_mut(ptr as *mut T, len)
        };

        Self { 0: data }
    }
}

impl<'a, T> Drop for Slice<'a, T> {
    fn drop(&mut self) {
        let size = self.0.len() * std::mem::size_of::<T>();
        unsafe {
            if libc::munmap(self.0.as_ptr() as *mut libc::c_void, size) != 0 {
                let err = std::ffi::CStr::from_ptr(libc::strerror(*libc::__errno_location()));
                panic!("SparseSlice: libc::munmap: {}", err.to_str().unwrap());
            }
        }
    }
}

#[test]
fn sparse_slice() {
    let width: usize = 4096;
    let size = width.pow(3);
    let slice = Slice::<usize>::new(size);

    for i in 0..128 {
        slice.0[i * 4096] = i;
    }

    for i in 0..128 {
        assert_eq!(slice.0[i * 4096], i);
    }

    for i in 128..256 {
        assert_eq!(slice.0[i * 4096], 0);
    }
}
