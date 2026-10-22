fn f()->u32{37}#[derive(Copy,Clone)]enum E{A(fn()->u32),Empty}union U{e:E,b:[u8;32]}const N:E=unsafe{(U{e:E::A(f)}).e};fn main(){check(match N{E::A(v)=>v()==37,_=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
