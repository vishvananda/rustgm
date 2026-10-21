use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);const X:u32=A.load(Ordering::Relaxed);
