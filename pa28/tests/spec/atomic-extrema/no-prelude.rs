#![no_implicit_prelude]
extern crate core as c;fn main(){let a=c::sync::atomic::AtomicU32::new(7);let _=a.fetch_min(3,c::sync::atomic::Ordering::Relaxed);let _=a.fetch_max(37,c::sync::atomic::Ordering::Release);}
