#[repr(align(16))]struct Z;const N:()={let a=Z;let p=&a as *const Z as *mut Z;unsafe{core::ptr::copy(&raw const a,p,1);}};fn main(){let _=N;}
