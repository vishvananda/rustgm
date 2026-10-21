use core::sync::atomic::AtomicU32;
static A:AtomicU32=AtomicU32::new(0);const R:&AtomicU32=&A;
