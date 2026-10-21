#![allow(invalid_reference_casting)]
const X:u32={let x=7u32;let s=&x;let p=(s as *const u32)as *mut u32;unsafe{let _r=&mut *p;}37};fn main(){let _=X;}
