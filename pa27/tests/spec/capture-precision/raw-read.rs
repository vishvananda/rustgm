fn shared<F:Fn()->u32>(_:&F){}fn main(){let mut x=7u32;let p=&raw const x;let c=||unsafe{*p};shared(&c);x=9;let _=c();}
