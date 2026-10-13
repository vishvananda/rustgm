fn id(x:&u8)->&u8{x}fn f(){let p:fn(_)->_=id;let _:for<'a>fn(&'a u8)->&'a u8=p;}
fn main() {}
