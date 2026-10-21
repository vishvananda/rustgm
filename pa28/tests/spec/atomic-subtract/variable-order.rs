use core::sync::atomic::{AtomicU32 as A,Ordering as O};fn f(a:&A,o:O)->u32{a.fetch_sub(3,o)+a.fetch_sub(4,o)}fn main(){let _=f(&A::new(1),O::AcqRel);}
