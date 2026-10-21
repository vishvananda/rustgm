const X:u32={let mut x=7u32;let p=&raw mut x as *const u32 as *mut u32;unsafe{core::ptr::write(p,37);x}};fn main(){let _=X;}
