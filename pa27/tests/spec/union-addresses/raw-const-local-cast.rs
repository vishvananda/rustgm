union U{a:u32,b:u8}const X:u32={let u=U{a:7};let p=(&raw const u.a) as *mut u32;unsafe{*p=9;*p}};fn main(){let _=X;}
