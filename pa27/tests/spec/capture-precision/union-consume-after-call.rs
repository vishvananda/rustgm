union U{a:u32,b:u8}fn consume(_:U){}fn main(){let u=U{a:7};let c=||unsafe{u.a};let _=c();consume(u);}
