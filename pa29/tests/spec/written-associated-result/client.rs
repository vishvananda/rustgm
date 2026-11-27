fn read_fused<T: base::Fused>(cursor: &T) -> T::Item {
    base::Read::read(cursor)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe(value: u8) -> u8 {
    let cursor = base::Cursor(&value, 0u8);
    *read_fused(&cursor)
}
