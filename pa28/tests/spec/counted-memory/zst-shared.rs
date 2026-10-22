#[repr(align(16))]struct Z;const N:()={let a=Z;let p=&a as *const Z as *mut Z;unsafe{core::ptr::copy_nonoverlapping(&raw const a,p,1);}};fn main(){let _=N;}
