fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call(f:&mut dyn FnMut(u8)->u8)->u8 { f(3) } fn main(){let mut n=4;let mut f=|x:u8|{n+=x;n};check(call(&mut f)==7);check(call(&mut f)==10);}
