union U{a:u32,b:u32}const X:u32={let p={let u=U{a:7};&raw const u.a};unsafe{let r=&*p;*r}};fn main(){let _=X;}
