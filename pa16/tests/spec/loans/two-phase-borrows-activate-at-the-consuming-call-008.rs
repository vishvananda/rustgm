fn take(_: &mut u8,_:u8){}fn f(p:&mut u8,c:bool){take(p,if c{*p}else{0});}
fn main() {}
