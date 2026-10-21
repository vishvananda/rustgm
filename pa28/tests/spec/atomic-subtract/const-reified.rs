use core::sync::atomic::{AtomicU32 as A,Ordering as O};const SWAP:fn(&A,u32,O)->u32=A::fetch_sub;const X:u32={let a=A::new(1);SWAP(&a,2,O::Relaxed)};fn main(){let _=X;}
