#[repr(C,packed)]struct P{a:u8,b:u32}const X:u32={let p=P{a:3,b:37};unsafe{core::ptr::read(&raw const p.b)}};fn main(){let _=X;}
