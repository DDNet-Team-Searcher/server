use libc::{
    c_char, c_void, ftruncate, mmap, off_t, shm_open, MAP_SHARED, O_CREAT, O_RDWR, PROT_READ,
    PROT_WRITE, S_IRUSR, S_IWUSR, S_IXUSR,
};
use std::ptr;

#[derive(Debug)]
pub struct SharedMemory {
    addr: *mut c_void,
}

//NOTE: internet told me to do this
unsafe impl Send for SharedMemory {}

impl SharedMemory {
    pub fn new(max: usize) -> Self {
        const STORAGE_ID: *const c_char = b"/ddts\0".as_ptr() as *const c_char;

        let (_, addr) = unsafe {
            let null = ptr::null_mut();
            let fd = shm_open(STORAGE_ID, O_CREAT | O_RDWR, S_IRUSR | S_IWUSR | S_IXUSR);

            ftruncate(fd, max as off_t);

            let addr = mmap(null, max, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);

            (fd, addr)
        };

        return Self { addr };
    }

    pub fn shutdown_server(&self, id: u32) {
        unsafe {
            *(self.addr.offset(id as isize) as *mut u8) = 1;
        };
    }
}
