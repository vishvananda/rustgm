fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call(f:&dyn Fn(u8)->u8)->u8 { f(3) } fn main(){let n=4;let f=|x:u8|x+n;check(call(&f)==7);}
