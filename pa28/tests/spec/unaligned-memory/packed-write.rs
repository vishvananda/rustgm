#[repr(C,packed)]struct P{a:u8,b:u32}const X:u32={let mut p=P{a:3,b:7};unsafe{core::ptr::write_unaligned(&raw mut p.b,37);}p.b};fn main(){let _=X;}
