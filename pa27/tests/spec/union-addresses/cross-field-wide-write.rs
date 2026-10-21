union U{a:u32,b:u8}const X:u8={let mut u=U{a:0};let b=&raw mut u.b;let a=&raw mut u.a;unsafe{*a=0x12345678;*b}};fn main(){let _=X;}
