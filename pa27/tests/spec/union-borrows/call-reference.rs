union U{a:u32,b:u32}const fn f<'a>(u:&'a U)->&'a u32{unsafe{&u.b}}const X:u32={let u=U{a:37};let r=f(&u);*r};fn main(){let _=X;}
