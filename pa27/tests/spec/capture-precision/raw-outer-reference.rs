struct S{p:*mut u32,free:u32}fn shared<F:Fn()->u32>(_:&F){}fn main(){let mut x=7u32;let s=S{p:&raw mut x,free:1};let r=&s;let c=||unsafe{*r.p=9;*r.p};shared(&c);let _=c();}
