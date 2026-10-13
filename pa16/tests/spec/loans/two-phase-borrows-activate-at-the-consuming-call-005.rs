fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(p,{let q=&*p;*q});}
fn main() {}
