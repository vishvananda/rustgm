fn g<'a:'a>(_:&'a u8){}fn f<'a>(x:&'a u8){let mut p=g::<'a>;p=g::<'static>;p(x)}
fn main() {}
