use core::sync::atomic::AtomicU32;const fn rest(a:&[AtomicU32])->&[AtomicU32]{match a{[r @ ..]=>r}}const R:&[AtomicU32]=rest(&[]);fn main(){let _=R;}
