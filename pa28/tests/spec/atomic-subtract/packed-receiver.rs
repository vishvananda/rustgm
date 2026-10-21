use core::sync::atomic::{AtomicU32 as A,Ordering as O};#[repr(packed)]struct P{tag:u8,a:A}fn main(){let p=P{tag:0,a:A::new(1)};let _=p.a.fetch_sub(2,O::Relaxed);}
