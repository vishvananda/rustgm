#[derive(Copy,Clone)]enum E<T>{A(T),Empty}union U<T:Copy>{e:E<T>,b:[u8;64]}const fn round<T:Copy>(e:E<T>)->E<T>{unsafe{(U{e}).e}}const N:bool=match(round(E::A(37u32)),round(E::A(true))){(E::A(a),E::A(b))=>a==37&&b,_=>false};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
