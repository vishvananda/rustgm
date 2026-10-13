fn id(x:&u8)->&u8{x}fn f<'a>(x:&'a u8)->&'a u8{let p:for<'b>fn(&'b u8)->&'b u8=id;p(x)}
fn main() {}
