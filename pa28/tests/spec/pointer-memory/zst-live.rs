#[derive(Copy,Clone)]#[repr(align(16))]struct Z;const X:Z={let mut z=Z;unsafe{let v=core::ptr::read(&raw const z);core::ptr::write(&raw mut z,v);z}};fn main(){let _=X;}
