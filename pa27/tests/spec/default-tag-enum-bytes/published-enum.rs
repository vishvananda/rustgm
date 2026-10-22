#[derive(Copy,Clone)]enum E{A(u32),Empty}union U{e:E,b:[u8;32]}const N:E=unsafe{(U{e:E::A(37)}).e};fn main(){check(match N{E::A(v)=>v==37,_=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
