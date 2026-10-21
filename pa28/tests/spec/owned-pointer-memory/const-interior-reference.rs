use core::sync::atomic::AtomicU32;static S:AtomicU32=AtomicU32::new(7);const R:&AtomicU32=unsafe{&*(&raw const S)};fn main(){let _=R;}
