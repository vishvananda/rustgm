use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(7);let _=a.compare_exchange_weak(7,9,O::Relaxed,O::Relaxed);}
