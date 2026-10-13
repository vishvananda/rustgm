fn f(p:for<'a>fn(&'a u8)->&'a u8){let _:fn(&'static u8)->&'static u8=p;}
fn main() {}
