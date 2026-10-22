const N:u32={let mut a=37u32;let p=&raw mut a;unsafe{core::ptr::copy(p,p,1);}a};fn main(){let _=N;}
