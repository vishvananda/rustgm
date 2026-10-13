fn g<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'a>(){let mut p=g::<'a>;p=g::<'static>;}
fn main() {}
