use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

// Raw libc symbols — left undefined in this .so, bound at dlopen() time against the
// ld-musl-x86_64.so.1 the host already mapped. There is no second libc in the process: these
// resolve straight into the host's musl.
//
// Signatures must match musl's exactly (open is variadic, read takes a void*), otherwise rustc's
// suspicious_runtime_symbol_definitions lint rejects them as conflicting with the ones std itself
// declares.
unsafe extern "C" {
    fn open(path: *const c_char, flags: c_int, ...) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn close(fd: c_int) -> c_int;
}

const O_RDONLY: c_int = 0;

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    // println! routes through libc's write() under std's hood — same dynamic resolution as the raw
    // calls below.
    println!("cdylib loaded, calling into musl...");

    unsafe {
        let path = CString::new("/etc/hostname").unwrap();
        let fd = open(path.as_ptr(), O_RDONLY);
        if fd < 0 {
            eprintln!("open() failed");
            return;
        }

        let mut buf = [0u8; 128];
        let n = read(fd, buf.as_mut_ptr() as *mut c_void, buf.len());
        close(fd);

        if n > 0 {
            let s = String::from_utf8_lossy(&buf[..n as usize]);
            println!("read from /etc/hostname: {}", s.trim());
        }
    }
}
