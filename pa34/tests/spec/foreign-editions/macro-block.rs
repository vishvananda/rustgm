#[unsafe(export_name = "edition_provider")]
pub extern "C" fn provide(x: i32) -> i32 { x + 1 }
macro_rules! declarations { () => { extern "C" { #[link_name = "edition_provider"] fn imported(x: i32) -> i32; } } }
declarations!();
fn main() { let value = unsafe { imported(6) }; let _ = 1u8 / ((value == 7) as u8); }
