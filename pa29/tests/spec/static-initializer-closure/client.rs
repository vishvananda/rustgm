extern crate base;

const _: [u8; 6] = [0; base::CLOSED as usize];

#[unsafe(no_mangle)]
pub extern "C" fn call_handler(value: u8) -> u8 {
    (base::table().handler)(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn address() -> usize {
    base::table() as *const base::Table as usize
}
