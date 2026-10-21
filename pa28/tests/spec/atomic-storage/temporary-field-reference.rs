use core::sync::atomic::AtomicU32;
const REFERENCE: &u32 = &(AtomicU32::new(0), 7).1;
