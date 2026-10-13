fn take(_: &mut u8,_:u8){}fn f(p:&mut u8,c:bool){take(if c{p}else{return},*p);}
fn main() {}
