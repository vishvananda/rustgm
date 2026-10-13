fn g<'a>()->&'a u8{loop{}}fn f<'a>(){let mut p=g::<'static>;p=g::<'a>;}
fn main() {}
