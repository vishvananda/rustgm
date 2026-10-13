fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(){let p=id as for<'b>fn(&'b u8)->&'b u8;}
fn main() {}
