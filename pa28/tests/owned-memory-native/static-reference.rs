use core::sync::atomic::{AtomicU32,Ordering};static S:AtomicU32=AtomicU32::new(7);const R:&AtomicU32=unsafe{&*(&raw const S)};fn main(){check(R.load(Ordering::Relaxed)==7);}
