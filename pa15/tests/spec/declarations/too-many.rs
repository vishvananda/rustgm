struct S<'a>(&'a u8);fn f(_:S<'static,'static>){}
fn main() {}
