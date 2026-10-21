use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(7);let _=a.compare_exchange(true,9,O::Relaxed,O::Relaxed);}
