union U{a:u32,b:u8}const X:u32={let mut u=U{a:0x12345678};let a=&raw mut u.a;let b=&raw mut u.b;unsafe{*b=0xab;*a}};fn main(){let _=X;}
