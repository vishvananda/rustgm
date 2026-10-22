#![allow(dead_code,unused_mut,invalid_reference_casting)]
use core::sync::atomic::AtomicU32;use core::mem::ManuallyDrop;union U{bits:u32,atomic:ManuallyDrop<AtomicU32>}const X:U=U{bits:7};
fn main(){let _=X;}
