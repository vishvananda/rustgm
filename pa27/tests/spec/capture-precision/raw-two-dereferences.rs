fn shared<F:Fn()->u32>(_:&F){}fn main(){let x=7u32;let p=&raw const x;let pp=&raw const p;let c=||unsafe{**pp};shared(&c);let _=c();}
