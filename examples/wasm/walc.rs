#![allow(dead_code)]
#![macro_use]

#[link(wasm_import_module = "walc")]
unsafe extern "C" {
    #[link_name = "print"]
    fn walc_print(ptr: *const u8, len: usize);
    #[link_name = "read"]
    fn walc_read(ptr: *mut u8, len: usize) -> usize;
}

pub fn print(message: &str) {
    unsafe { walc_print(message.as_ptr(), message.len()) }
}

pub fn read(buffer: &mut [u8]) -> usize {
    unsafe { walc_read(buffer.as_mut_ptr(), buffer.len()) }
}

pub fn read_byte() -> Option<u8> {
    let mut buf = [0u8; 1];
    match read(&mut buf) {
        0 => None,
        _ => Some(buf[0]),
    }
}

pub fn read_all() -> Vec<u8> {
    let mut data = Vec::new();
    loop {
        let mut buf = [0u8; 1024];
        let len = read(&mut buf);
        if len == 0 {
            break;
        }

        data.extend_from_slice(&buf[..len]);
    }

    data
}

pub fn read_string() -> Result<String, std::string::FromUtf8Error> {
    let data = read_all();
    String::from_utf8(data)
}

pub fn read_line() -> Result<String, std::string::FromUtf8Error> {
    let mut line = Vec::new();
    loop {
        match read_byte() {
            Some(b'\n') => break,
            Some(byte) => line.push(byte),
            None => break,
        }
    }
    String::from_utf8(line)
}

macro_rules! print {
    ($($arg:tt)*) => ({
        let s = format!($($arg)*);
        print(&s);
    })
}

macro_rules! println {
    () => (print("\n"));
    ($($arg:tt)*) => ({
        print!($($arg)*);
        println!();
    })
}
