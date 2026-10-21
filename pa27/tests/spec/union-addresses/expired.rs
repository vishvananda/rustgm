union U{a:u32,b:u8}const X:u32={let p={let u=U{a:7};&raw const u.a};unsafe{*p}};fn main(){let _=X;}
