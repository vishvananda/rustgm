macro_rules! make { ($name:ident) => { #[$name] pub extern "C" fn value() -> u8 { 7 } } }
make!(no_mangle);
fn main() { let _ = 1u8 / ((value() == 7) as u8); }
