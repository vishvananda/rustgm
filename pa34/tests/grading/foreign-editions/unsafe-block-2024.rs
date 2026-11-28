#[unsafe(export_name = "edition_provider")]
pub extern "C" fn provide(x: i32) -> i32 { x + 1 }
unsafe extern "C" { #[link_name = "edition_provider"] fn imported(x: i32) -> i32; }
fn main() { let value = unsafe { imported(6) }; let _ = 1u8 / ((value == 7) as u8); }
