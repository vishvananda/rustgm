union U{a:u32,b:u32}const X:&u32={let u=U{a:7};let p=&raw const u.a;unsafe{&*p}};fn main(){let _=X;}
