#[derive(Copy,Clone)]#[repr(align(16))]struct Z;const X:Z=unsafe{core::ptr::read(1usize as *const Z)};fn main(){let _=X;}
