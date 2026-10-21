fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<F:Fn(u8)->u8>(f:F)->u8{f(2)}fn main(){let n=7u8;check(invoke(|x|x+n)==9);}
