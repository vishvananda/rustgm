#[unsafe(export_name = "edition_value")]
pub static VALUE: u8 = 7;
extern "C" { #[link_name = "edition_value"] static IMPORTED: u8; }
fn main() { let value = unsafe { IMPORTED }; let _ = 1u8 / ((value == 7) as u8); }
