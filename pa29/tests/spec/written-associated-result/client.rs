#[unsafe(no_mangle)]
pub extern "C" fn observe(value: u8) -> u8 {
    let cursor = base::Cursor(&value, 0u8);
    *base::Read::read(&cursor)
}
