union U{a:u32,b:u8}fn main(){let u=U{a:7};let outer=||{let c=||unsafe{u.a};c()};let _=outer();}
