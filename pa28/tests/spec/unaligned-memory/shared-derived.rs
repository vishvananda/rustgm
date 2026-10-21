const X:u32={let x=7u32;let p=&x as *const u32 as *mut u32;unsafe{core::ptr::write_unaligned(p,37);}x};fn main(){let _=X;}
