use core::sync::atomic::AtomicU32;static MARKER:AtomicU32=AtomicU32::new(0);const fn tail(a:&[u8])->&[u8]{match a{[_,r @ ..]=>r,_=>a}}const R:&[u8]=tail(b"abc");const N:u8=R[0];fn main(){let _=N;}
