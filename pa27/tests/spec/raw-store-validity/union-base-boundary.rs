union U{a:u32,b:bool}const X:u32={let mut u=U{a:7};let p=&raw mut u;unsafe{(*p).a=9;u.a}};fn main(){let _=X;}
