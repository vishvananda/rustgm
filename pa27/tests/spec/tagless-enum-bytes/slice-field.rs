static S:[u8;2]=[17,20];#[derive(Copy,Clone)]enum E{Only(&'static[u8])}union U{e:E,b:[u8;16]}const N:u8=unsafe{match(U{e:E::Only(&S)}).e{E::Only(v)=>v[0]+v[1]}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
