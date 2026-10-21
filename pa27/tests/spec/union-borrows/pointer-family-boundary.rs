union U{a:u32,p:*const u32}const X:u32={let u=U{a:37};unsafe{let r=&u.a;*r}};fn main(){let _=X;}
