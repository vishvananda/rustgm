const X:u32={let mut x=0u32;let p=(&raw mut x) as *mut u8;unsafe{*p=9;}x};fn main(){let _=X;}
