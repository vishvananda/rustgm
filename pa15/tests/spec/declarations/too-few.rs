struct S<'a,'b>(&'a u8,&'b u8);fn f(_:S<'static>){}
fn main() {}
