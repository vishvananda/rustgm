union U{a:u32,b:u8}fn main(){let u=U{a:7};let c=||unsafe{&raw const u.a};let _=c();}
