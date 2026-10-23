#[no_mangle]
pub extern "C" fn edition_value() -> u8 { 7 }
fn main() { let _ = 1u8 / ((edition_value() == 7) as u8); }
