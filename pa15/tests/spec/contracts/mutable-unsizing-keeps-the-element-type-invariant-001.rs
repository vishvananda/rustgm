fn f<'a:'b,'b>(x:&'b [&'a u8;1])->&'b [&'b u8]{x}
fn main() {}
