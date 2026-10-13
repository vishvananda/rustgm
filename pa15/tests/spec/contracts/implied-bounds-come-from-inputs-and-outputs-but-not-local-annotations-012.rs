fn f<'a,'b>(x:&'a *const &'b u8,y:&'b u8)->&'a u8{y}
fn main() {}
