use core::sync::atomic::{AtomicU32 as A,Ordering as O};const fn f(a:&A)->u32{if false{let _=a.compare_exchange_weak(7,9,O::Relaxed,O::Relaxed);}0}fn main(){}
