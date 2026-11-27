pub fn unsupported(value: &u8) {
    let cursor = base::Cursor(value, false);
    let _ = base::Read::read(&cursor);
}
