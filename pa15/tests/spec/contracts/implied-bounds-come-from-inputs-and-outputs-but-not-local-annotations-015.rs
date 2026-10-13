struct S<'a,'b>(fn(&'a &'b u8));fn f<'a,'b>(x:S<'a,'b>,y:&'b u8)->&'a u8{y}
fn main() {}
