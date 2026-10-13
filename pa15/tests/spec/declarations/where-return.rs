fn f<'a,'b>(x:&'a u8)->&'b u8 where 'a:'b{x}
fn main() {}
