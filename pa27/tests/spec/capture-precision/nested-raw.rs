fn shared<F:Fn()->u32>(_:&F){}fn main(){let mut x=7u32;let p=&raw mut x;let outer=||{let c=||unsafe{*p=9;*p};shared(&c);c()};shared(&outer);let _=outer();}
