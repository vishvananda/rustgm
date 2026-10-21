#![allow(invalid_reference_casting)]
const X:u32={let x=7u32;let p=&x as *const u32 as *mut u32;let mut q=p;unsafe{let r=core::ptr::read(&raw const q);core::ptr::write(&raw mut q,r);core::ptr::write(q,37);x}};fn main(){let _=X;}
