#[export_name = "edition_value"]
pub extern "C" fn value() -> u8 { 7 }
fn main() { let _ = 1u8 / ((value() == 7) as u8); }
