use core::sync::atomic::{AtomicU32 as A,Ordering as O};const X:u32={let a=A::new(1);unsafe{a.fetch_nand(1,O::Relaxed)}};fn main(){let _=X;}
