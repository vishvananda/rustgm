union U{a:u32,b:u32}const X:u32={let mut u=U{a:7};let r=&mut u.a;*r=37;unsafe{u.b}};fn main(){let _=X;}
