union U{a:u32,b:u8}fn main(){let mut u=U{a:7};let mut c=||&raw mut u.a;let _=c();}
