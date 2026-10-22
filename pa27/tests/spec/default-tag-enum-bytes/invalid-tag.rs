#[derive(Copy,Clone)]enum E{A=3,B=7}union U{e:E,b:[u8;16]}const N:E=unsafe{(U{b:[255;16]}).e};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
