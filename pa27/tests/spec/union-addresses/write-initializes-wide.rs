union U{a:u32,b:u8}const X:u32={let mut u=U{b:1};let p=&raw mut u.a;unsafe{*p=37;*p}};fn main(){let _=X;}
