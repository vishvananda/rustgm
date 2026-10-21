fn main(){let x=7u32;let r=&x;let p=r as *const u32 as *mut u32;unsafe{*p=9;}}
