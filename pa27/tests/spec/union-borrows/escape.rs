union U{a:u32,b:u32}const R:&u32={let u=U{a:37};unsafe{&u.a}};fn main(){let _=R;}
