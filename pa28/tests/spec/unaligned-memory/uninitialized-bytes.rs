union U{a:u8,b:u32}const X:u32={let u=U{a:7};unsafe{core::ptr::read_unaligned(&raw const u.b)}};fn main(){let _=X;}
