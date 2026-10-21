use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(7);let _=A::compare_exchange(&a,7,9,O::SeqCst,O::Release);}
