macro_rules! inner { ($a:meta) => { #[$a] pub extern "C" fn value() -> u8 { 7 } } }
macro_rules! outer { ($a:meta) => { inner!($a); } }
outer!(no_mangle);
fn main() { let _ = 1u8 / ((value() == 7) as u8); }
