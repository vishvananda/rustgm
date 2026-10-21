static N:u32=37;#[repr(C,packed)]struct P{a:u8,r:&'static u32}const X:&u32={let p=P{a:3,r:&N};unsafe{core::ptr::read_unaligned(&raw const p.r)}};fn main(){let _=X;}
