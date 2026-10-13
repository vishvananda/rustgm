fn f(p:for<'a>fn(&'a &'a u8)){let q:for<'a>fn(&'a _)=p;}
fn main() {}
