macro_rules! make { () => { #[cfg(target_has_atomic)] mod active { #[cfg(target_has_atomic_load_store)] fn required() {} } } } make!(); fn main() {}
