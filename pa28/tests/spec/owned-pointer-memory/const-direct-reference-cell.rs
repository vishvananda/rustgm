const X:u32={let mut n=7u32;let mut r=&mut n;let p=&raw mut r;unsafe{let q=core::ptr::read(p);*q=37;core::ptr::write(p,q);}*r};fn main(){let _=X;}
