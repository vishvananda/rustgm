use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(0);fn write(o:Ordering){A.store(7,o);}fn main(){write(Ordering::Acquire);}
