use core::sync::atomic::{AtomicU32,Ordering};#[repr(C,packed)]struct P{a:u8,b:AtomicU32}fn main(){let p=P{a:0,b:AtomicU32::new(7)};let _=p.b.fetch_min(3,Ordering::Relaxed);}
