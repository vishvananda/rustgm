use core::sync::atomic::AtomicU32;const fn tail(a:&[AtomicU32])->&[AtomicU32]{match a{[_,r @ ..]=>r,_=>a}}const R:&[AtomicU32]=tail(&[AtomicU32::new(0)]);fn main(){let _=R;}
