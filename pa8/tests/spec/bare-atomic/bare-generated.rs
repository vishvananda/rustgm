macro_rules! make { () => { #[cfg(target_has_atomic)] fn helper() -> u32 { 7 } } } make!(); fn main() { let _ = helper(); }
