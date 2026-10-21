const S:u32=37;const X:u32={let mut p=&S as *const u32;unsafe{let q=core::ptr::read(&raw const p);core::ptr::write(&raw mut p,q);*p}};fn main(){let _=X;}
