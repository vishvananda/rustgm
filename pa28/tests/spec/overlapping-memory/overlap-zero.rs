const N:u32={let mut a=7u32;let p=&raw mut a;unsafe{core::ptr::copy(p,p,0);}a};fn main(){let _=N;}
