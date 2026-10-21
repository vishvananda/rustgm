fn shared<F:Fn()->*mut u32>(_:&F){}fn main(){let mut x=7u32;let p=&raw mut x;let c=||unsafe{&raw mut *p};shared(&c);let _=c();}
