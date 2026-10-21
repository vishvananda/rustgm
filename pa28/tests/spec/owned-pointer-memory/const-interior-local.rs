use core::sync::atomic::AtomicU32;const X:AtomicU32={let a=AtomicU32::new(7);unsafe{core::ptr::read(&raw const a)}};fn main(){let _=X;}
