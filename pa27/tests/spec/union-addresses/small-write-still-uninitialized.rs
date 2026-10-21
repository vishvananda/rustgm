union U{a:u32,b:u8}const X:u32={let mut u=U{b:1};let a=&raw const u.a;let b=&raw mut u.b;unsafe{*b=2;*a}};fn main(){let _=X;}
