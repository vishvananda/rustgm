use core::sync::atomic::{AtomicU32 as A,Ordering as O};use core::result::Result as R;fn f(a:&A)->R<u32,u32>{a.compare_exchange(7,9,O::Relaxed,O::Acquire)}fn main(){let _=f(&A::new(7));}
