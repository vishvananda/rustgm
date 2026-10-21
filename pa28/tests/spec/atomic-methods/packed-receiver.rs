use core::sync::atomic::{AtomicU32,Ordering};#[repr(packed)]struct P(u8,AtomicU32);fn f(p:&P)->u32{p.1.load(Ordering::Relaxed)}
