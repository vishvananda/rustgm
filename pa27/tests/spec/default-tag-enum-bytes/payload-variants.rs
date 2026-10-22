#[derive(Copy,Clone)]enum E{A(u32),B(bool),Empty}union U{e:E,b:[u8;32]}const N:bool=unsafe{let a=match(U{e:E::A(37)}).e{E::A(v)=>v==37,_=>false};let b=match(U{e:E::B(true)}).e{E::B(v)=>v,_=>false};a&&b};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
