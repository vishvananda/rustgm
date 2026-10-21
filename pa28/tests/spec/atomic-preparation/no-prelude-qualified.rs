#![no_implicit_prelude]extern crate core;fn main(){let a=core::sync::atomic::AtomicU32::new(7);let _=a.load(core::sync::atomic::Ordering::SeqCst);}
