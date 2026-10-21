fn main(){let p:&dyn Fn()->u8;{let x=7u8;let f=||x;p=&f;}let _=p();}
