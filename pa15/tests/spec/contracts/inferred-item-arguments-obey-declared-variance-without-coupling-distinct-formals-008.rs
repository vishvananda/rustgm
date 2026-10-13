fn g<'a>()->&'a u8{loop{}}fn f<'a>(){let mut p=g::<'a>;p=g::<'static>;}
fn main() {}
