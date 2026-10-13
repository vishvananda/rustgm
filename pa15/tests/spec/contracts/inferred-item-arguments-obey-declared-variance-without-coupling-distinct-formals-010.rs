fn g<'a>()->&'a u8{loop{}}fn f<'a>()->&'static u8{let mut p=g::<'static>;p=g::<'a>;p()}
fn main() {}
