#[repr(packed)]struct P{a:u8,b:u32}const X:u32={let mut p=P{a:1,b:7};let r=&raw mut p.b;unsafe{*r=9;}p.b};fn main(){let _=X;}
