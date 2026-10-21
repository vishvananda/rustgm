fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add(x:u8)->u8{x+4} fn call(f:&dyn Fn(u8)->u8)->u8{f(3)} fn main(){let f:fn(u8)->u8=add;check(call(&f)==7);}
