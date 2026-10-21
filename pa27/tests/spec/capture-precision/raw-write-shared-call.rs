fn shared<F:Fn()->u32>(_:&F){}fn main(){let mut x=7u32;let p=&raw mut x;let c=||unsafe{*p=9;*p};shared(&c);let _=c();let _=c();}
