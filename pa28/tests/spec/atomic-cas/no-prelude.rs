#![no_implicit_prelude]
extern crate core;fn main(){let a=core::sync::atomic::AtomicU32::new(7);let _=a.compare_exchange(7,9,core::sync::atomic::Ordering::Relaxed,core::sync::atomic::Ordering::Relaxed);}
