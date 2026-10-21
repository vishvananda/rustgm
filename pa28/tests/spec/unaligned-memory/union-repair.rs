union U{a:u8,b:u32}const X:u32={let mut u=U{a:7};unsafe{core::ptr::write_unaligned(&raw mut u.b,37);core::ptr::read_unaligned(&raw const u.b)}};fn main(){let _=X;}
