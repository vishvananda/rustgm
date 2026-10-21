fn main(){let mut x=1u8;let f=||x;let p:&dyn Fn()->u8=&f;x=2;let _=p();}
