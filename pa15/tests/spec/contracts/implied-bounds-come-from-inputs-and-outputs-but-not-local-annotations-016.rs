struct C<'b>(fn(&'b u8));fn f<'a,'b>(x:&'a C<'b>,y:&'b u8)->&'a u8{y}
fn main() {}
