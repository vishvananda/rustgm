macro_rules! make { ($flag:meta) => { #[cfg($flag)] fn helper() -> u32 { 7 } } } make!(target_has_atomic); fn main() { let _ = helper(); }
