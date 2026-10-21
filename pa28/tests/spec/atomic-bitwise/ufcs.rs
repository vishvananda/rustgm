use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(1);let _=A::fetch_and(&a,2,O::Release);let _=A::fetch_and(&a,3,O::Acquire);}
