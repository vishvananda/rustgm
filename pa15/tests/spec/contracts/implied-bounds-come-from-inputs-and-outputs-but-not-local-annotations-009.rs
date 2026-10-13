fn f<'a,'b>(x:for<'c>fn(&'c &'b u8),y:&'b u8)->&'a u8{y}
fn main() {}
