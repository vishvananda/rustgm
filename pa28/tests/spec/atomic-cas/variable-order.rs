use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn f(a:&A,s:O,f:O)->Result<u32,u32>{a.compare_exchange_weak(7,9,s,f)}fn main(){let _=f(&A::new(7),O::Release,O::Acquire);}
