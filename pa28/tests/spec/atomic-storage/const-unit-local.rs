use core::sync::atomic::AtomicU32;
const VALUE: () = { let value = AtomicU32::new(0); let _reference = &value; };
