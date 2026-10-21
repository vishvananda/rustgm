fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke(f:&mut dyn FnMut(u8)->u8)->u8{f(2)}fn main(){let mut n=7u8;check(invoke(&mut |x|{n+=x;n})==9);check(n==9);}
