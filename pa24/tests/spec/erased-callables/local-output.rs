fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call(f:&dyn Fn()->&'static u8)->&'static u8{f()} fn main(){let n=7u8;let f=||&n;call(&f);}
