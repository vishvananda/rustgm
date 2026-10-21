fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<F:for<'a>FnMut(&'a u8)->&'a u8>(mut f:F)->u8{let n=7u8;*f(&n)}fn main(){let marker=0u8;let f=|x:&u8|{let _=marker;x};check(call(f)==7);}
