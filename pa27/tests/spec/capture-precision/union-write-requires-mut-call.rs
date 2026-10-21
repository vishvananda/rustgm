union U{a:u32,b:u8}fn main(){let mut u=U{a:7};let c=||{u.a=9;};c();}
