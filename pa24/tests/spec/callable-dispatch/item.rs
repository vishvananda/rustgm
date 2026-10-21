fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u8)->u8{x+2}fn call(f:&dyn Fn(u8)->u8)->u8{f(7)}fn main(){check(call(&inc)==9);}
