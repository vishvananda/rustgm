#[derive(Copy,Clone)]enum E{A=-3,B=7}union U{e:E,b:[u8;16]}const N:i8=unsafe{match(U{e:E::A}).e{E::A=>-3,E::B=>7}};fn main(){check(N== -3)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
