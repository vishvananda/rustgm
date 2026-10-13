struct S<'a>(&'a u8);fn f<'a>(){let mut c=S::<'a>;c=S::<'static>;}
fn main() {}
