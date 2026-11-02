#![feature(cfg_target_has_atomic)] #[cfg(target_has_atomic)] mod active { #[cfg(target_has_atomic_load_store)] pub fn helper() -> u32 { 7 } } fn main() { let _ = active::helper(); }
