fn g<'a:'static>(_:&'a u8){}fn f<'a>(){let mut p=g::<'static>;p=g::<'a>;}
fn main() {}
