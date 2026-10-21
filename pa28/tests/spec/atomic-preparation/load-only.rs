use core::sync::atomic::AtomicU32;static A:AtomicU32=AtomicU32::new(7);fn main(){let _=core::sync::atomic::AtomicU32::load(&A,core::sync::atomic::Ordering::SeqCst);}
