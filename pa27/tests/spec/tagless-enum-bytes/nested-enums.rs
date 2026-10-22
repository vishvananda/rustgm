#[derive(Copy,Clone)]enum F{Only(u32)}#[derive(Copy,Clone)]enum E{Only(F)}union U{e:E,b:[u8;4]}const N:u32=unsafe{match(U{e:E::Only(F::Only(37))}).e{E::Only(F::Only(v))=>v}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
