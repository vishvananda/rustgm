use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(1);let fetch_sub:unsafe fn(&A,u32,O)->u32=A::fetch_sub;let _=unsafe{fetch_sub(&a,2,O::SeqCst)};}
