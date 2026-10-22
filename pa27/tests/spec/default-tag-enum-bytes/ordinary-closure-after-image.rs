#[derive(Copy,Clone)]enum V{A(u32),Empty}union U{e:V,b:[u8;32]}const M:u32=unsafe{match(U{e:V::A(17)}).e{V::A(v)=>v,_=>0}};enum E<T>{A(T),Empty}const N:u32={let e=E::A(||37u32);match e{E::A(f)=>{let _=f;M+20},E::Empty=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
