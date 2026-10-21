use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn f(a:&A,s:O,f:O){let _=a.compare_exchange(7,9,s,f);}fn main(){f(&A::new(7),O::SeqCst,O::Release);}
