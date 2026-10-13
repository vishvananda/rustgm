fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(){let _:for<'b>fn(&'b u8)->&'b u8=id;}
fn main() {}
