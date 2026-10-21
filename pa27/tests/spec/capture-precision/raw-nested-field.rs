fn shared<F:Fn()->u32>(_:&F){}struct S{p:*mut u32,free:u32}fn main(){let mut x=7u32;let mut s=S{p:&raw mut x,free:1};let c=||unsafe{*s.p=9;*s.p};shared(&c);s.free=3;let _=c();}
