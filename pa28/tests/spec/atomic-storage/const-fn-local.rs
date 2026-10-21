use core::sync::atomic::AtomicU32;
const fn make() -> AtomicU32 { let value = AtomicU32::new(0); { let _reference = &value; } value }
const VALUE: AtomicU32 = make();
