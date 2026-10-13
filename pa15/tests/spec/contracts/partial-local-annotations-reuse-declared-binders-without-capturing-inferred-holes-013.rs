fn f<'a>(p:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{let q:fn(&'a _)->&'a _=p;q(x)}
fn main() {}
