const X:u32={let mut a=[7u32,11,19];unsafe{let b=core::ptr::read(&raw const a);core::ptr::write(&raw mut a,[b[2],b[0],b[1]]);a[0]+a[1]+a[2]}};fn main(){let _=X;}
