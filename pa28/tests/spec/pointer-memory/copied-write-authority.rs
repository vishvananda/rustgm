const X:u32={let mut x=7u32;let p=&raw mut x;let mut q=p;unsafe{let r=core::ptr::read(&raw const q);core::ptr::write(&raw mut q,r);*q=37;x}};fn main(){let _=X;}
