fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<F:for<'a>Fn(&'a u8)->&'a u8>(f:F)->u8{let x=7u8;*f(&x)}fn main(){check(invoke(|x|x)==7);}
