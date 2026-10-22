fn main(){let a=7u32;let p=&a as *const u32 as *mut u32;unsafe{core::ptr::copy_nonoverlapping(&raw const a,p,0);}check(a==7);}
