use core::sync::atomic::{AtomicU32 as A,Ordering as O};const X:Result<u32,u32>={let a=A::new(7);a.compare_exchange(7,9,O::Relaxed,O::Relaxed)};fn main(){let _=X;}
