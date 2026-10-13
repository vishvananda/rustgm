struct S<'a,'b>(&'a &'b u8);fn f<'a,'b>(x:&'b u8){let y:S<'a,'b>;let z:&'a u8=x;}
fn main() {}
