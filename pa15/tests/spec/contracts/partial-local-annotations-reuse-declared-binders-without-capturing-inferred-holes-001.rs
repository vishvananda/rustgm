fn g(x:&u8){}fn f(){let p:fn(_)=g;let _:for<'a>fn(&'a u8)=p;}
fn main() {}
