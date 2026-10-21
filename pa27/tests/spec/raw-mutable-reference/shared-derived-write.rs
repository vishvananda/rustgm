#![allow(invalid_reference_casting)]
const X:u32={let x=7u32;let p=(&x as *const u32)as *mut u32;unsafe{let r=&mut *p;*r=37;}x};fn main(){let _=X;}
