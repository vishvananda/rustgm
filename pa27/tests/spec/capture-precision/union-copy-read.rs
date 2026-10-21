union U{a:u32,b:u8}fn shared<F:Fn()->u32>(_:&F){}fn main(){let u=U{a:7};let c=||unsafe{u.a};shared(&c);let _=c();let _=c();}
