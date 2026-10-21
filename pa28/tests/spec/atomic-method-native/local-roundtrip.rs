use core::sync::atomic::{AtomicU32,Ordering};fn main(){let a=AtomicU32::new(7);a.store(42,Ordering::Relaxed);check(a.load(Ordering::Relaxed)==42);}
