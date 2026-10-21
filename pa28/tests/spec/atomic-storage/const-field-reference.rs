use core::sync::atomic::AtomicU32;
const PAIR: (AtomicU32, u32) = (AtomicU32::new(0), 7);
const REFERENCE: &u32 = &PAIR.1;
