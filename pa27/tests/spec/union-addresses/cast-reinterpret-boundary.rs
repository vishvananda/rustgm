union U{a:u32,b:u8}const X:u8={let u=U{a:7};let p=(&raw const u.a) as *const u8;unsafe{*p}};fn main(){let _=X;}
