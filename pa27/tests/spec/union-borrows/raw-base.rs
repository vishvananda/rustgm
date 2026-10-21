union U{a:u32,b:u32}const X:u32={let u=U{a:37};let p=&raw const u;unsafe{let r=&(*p).a;*r}};fn main(){let _=X;}
