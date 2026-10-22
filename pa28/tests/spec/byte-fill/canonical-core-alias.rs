#![allow(dead_code,unused_mut)]
#![no_implicit_prelude]extern crate core as library;const X:u32={let mut x=0u32;unsafe{library::ptr::write_bytes(&raw mut x,1,1);}x};fn main(){let _=X;}
