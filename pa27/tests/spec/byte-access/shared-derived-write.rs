const N:u32={let x=7u32;let p=&x as *const u32 as *mut u8;unsafe{*p=1;}x};fn main(){}
