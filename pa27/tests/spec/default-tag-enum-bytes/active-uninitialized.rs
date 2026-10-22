#[derive(Copy,Clone)]enum E{A(u64),Empty}union U{e:E,b:[u8;1]}const N:E=unsafe{(U{b:[0]}).e};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
