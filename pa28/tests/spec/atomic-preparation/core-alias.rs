#![no_implicit_prelude]extern crate core as c;fn main(){let a=c::sync::atomic::AtomicU32::new(7);let _=a.load(c::sync::atomic::Ordering::SeqCst);}
