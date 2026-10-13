fn f(p:&for<'a>fn(&'a u8)->&'a u8){let q:&fn(_)->_=p;}
fn main() {}
