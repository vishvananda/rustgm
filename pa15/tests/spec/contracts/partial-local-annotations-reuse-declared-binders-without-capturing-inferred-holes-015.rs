fn f(p:for<'a,'b>fn(&'a u8,&'b u8)){let q:for<'a>fn(_,&'a _)=p;}
fn main() {}
