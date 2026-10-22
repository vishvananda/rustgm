static S:[u8;3]=[17,20,9];#[derive(Copy,Clone)]enum E{A(&'static[u8]),B(u64),Empty}union U{e:E,b:[u8;64]}const N:u8=unsafe{match(U{e:E::A(&S)}).e{E::A(v)=>v[0]+v[1],_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
