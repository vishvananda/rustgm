#![feature(concat_bytes)]
#[unsafe(no_mangle)]pub extern "C" fn answer()->*const u8{concat_bytes!(b"AB") as *const [u8;2] as *const u8}
