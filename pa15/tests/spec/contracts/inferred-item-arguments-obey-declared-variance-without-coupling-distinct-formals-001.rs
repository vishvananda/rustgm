fn g<'a:'a>(_:&'a u8){}fn f<'a>(){let mut p=g::<'a>;p=g::<'static>;}
fn main() {}
