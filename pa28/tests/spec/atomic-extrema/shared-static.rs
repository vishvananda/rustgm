use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(7);fn main(){let _=A.fetch_min(3,Ordering::Relaxed);let _=A.fetch_max(37,Ordering::Acquire);}
