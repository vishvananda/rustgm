#[repr(packed)]struct P{a:u8,b:u32}const X:u8={let mut p=P{a:1,b:7};let r=&raw mut p.a;unsafe{*r=9;}p.a};fn main(){let _=X;}
