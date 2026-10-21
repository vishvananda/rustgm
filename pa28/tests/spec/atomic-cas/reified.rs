use core::sync::atomic::{AtomicU32 as A,Ordering as O};const F:fn(&A,u32,u32,O,O)->Result<u32,u32>=A::compare_exchange;fn main(){let a=A::new(7);let _=F(&a,7,9,O::Relaxed,O::SeqCst);}
