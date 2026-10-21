const X:u32={let p={let mut x=37u32;&raw mut x};unsafe{core::ptr::write_unaligned(p,7);}0};fn main(){let _=X;}
