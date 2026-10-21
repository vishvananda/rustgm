#[derive(Clone,Copy)]union U{a:u32,b:u8}const X:(u32,u32)={let mut u=U{a:0x12345678};let v=u;let a=&raw const u.a;let b=&raw mut u.b;unsafe{*b=0xab;(*a,v.a)}};fn main(){let _=X;}
