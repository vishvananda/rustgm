fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<F:FnMut(u8)->u8>(mut f:F)->u8{f(2)+f(3)}fn main(){let mut n=7u8;check(invoke(|x|{n+=x;n})==21);check(n==12);}
