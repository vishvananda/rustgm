use core::sync::atomic::{AtomicU32 as A,Ordering as O};enum Result<T,E>{Ok(T),Err(E)}fn f(a:&A)->Result<u32,u32>{a.compare_exchange(7,9,O::Relaxed,O::Relaxed)}fn main(){let _=f(&A::new(7));}
