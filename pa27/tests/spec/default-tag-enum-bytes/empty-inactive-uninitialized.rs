#[derive(Copy,Clone)]enum E{A(u64),Empty}union U{e:E,b:[u8;8]}const N:bool=unsafe{match(U{b:[1,0,0,0,0,0,0,0]}).e{E::Empty=>true,_=>false}};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
