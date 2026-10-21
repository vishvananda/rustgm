const X:u32={let mut x=7u32;let p=(&raw mut x) as *const u32 as *mut u32;unsafe{*p=9;}x};fn main(){let _=X;}
