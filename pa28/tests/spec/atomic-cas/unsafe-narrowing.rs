use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn main(){let a=A::new(7);let f:unsafe fn(&A,u32,u32,O,O)->Result<u32,u32>=A::compare_exchange;let _=unsafe{f(&a,7,9,O::Relaxed,O::Acquire)};}
