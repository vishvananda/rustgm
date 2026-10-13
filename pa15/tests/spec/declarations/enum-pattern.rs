enum E<'a>{A(&'a u8),B}fn f<'a>(x:E<'a>,y:&'a u8)->&'a u8{match x{E::A(v)=>v,E::B=>y}}
fn main() {}
