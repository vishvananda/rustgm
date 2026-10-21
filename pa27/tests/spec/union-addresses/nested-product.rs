union U{a:u32,b:u8}struct S{pad:u8,u:U}const X:u32={let mut s=S{pad:1,u:U{a:0x12345678}};let a=&raw const s.u.a;let b=&raw mut s.u.b;unsafe{*b=0xab;*a}};fn main(){let _=X;}
