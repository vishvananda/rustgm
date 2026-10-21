use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(1);let _=a.swap::<u32>(2,O::Relaxed);}
