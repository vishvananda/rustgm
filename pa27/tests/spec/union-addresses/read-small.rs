union U{a:u32,b:u8}const X:u8={let u=U{a:0x12345678};let p=&raw const u.b;unsafe{*p}};fn main(){let _=X;}
