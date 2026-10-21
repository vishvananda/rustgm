#![allow(invalid_reference_casting)]
const fn pass(p:*mut u32)->*mut u32{p}const X:u32={let x=7u32;let s=&x;let pair=((s as *const u32)as *mut u32,0u32);let p=pass(pair.0);unsafe{let _r=&mut *p;}37};fn main(){let _=X;}
