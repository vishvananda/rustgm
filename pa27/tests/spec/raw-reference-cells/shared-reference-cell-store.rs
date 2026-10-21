#![allow(invalid_reference_casting)]
const X:u32={let x=7u32;let y=37u32;let r=&x;let p=(&r as *const &u32)as *mut &u32;unsafe{*p=&y;}*r};fn main(){let _=X;}
