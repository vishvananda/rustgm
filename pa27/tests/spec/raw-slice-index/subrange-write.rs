#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let mut a=[3u32,5,7];let [_,ref mut rest @ ..]=a;let r:&mut [u32]=rest;let p=r as *mut [u32];unsafe{(*p)[1]=41;}a[2]};fn main(){let _=X;}
