use core::sync::atomic::{AtomicU32 as A,Ordering as O};const X:Result<u32,u32>={let a=A::new(7);let f=A::compare_exchange_weak;f(&a,7,9,O::Relaxed,O::Relaxed)};fn main(){let _=X;}
