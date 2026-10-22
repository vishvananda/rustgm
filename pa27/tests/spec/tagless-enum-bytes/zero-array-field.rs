#[derive(Copy,Clone)]enum E{Only([u8;0])}union U{e:E,b:[u8;0]}const N:bool=unsafe{match(U{e:E::Only([])}).e{E::Only(v)=>match v{[]=>true}}};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
