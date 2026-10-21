use core::sync::atomic::{AtomicU32,Ordering};#[repr(packed)]struct P(u8,AtomicU32);fn f(p:&P)->*const AtomicU32{&raw const p.1}
