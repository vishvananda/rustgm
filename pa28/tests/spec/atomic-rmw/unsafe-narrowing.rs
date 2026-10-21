use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(1);let swap:unsafe fn(&A,u32,O)->u32=A::swap;let _=unsafe{swap(&a,2,O::SeqCst)};}
