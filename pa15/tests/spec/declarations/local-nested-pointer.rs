fn f<'a>(p:for<'b>fn(&'b u8,&'a u8)->&'b u8,x:&'a u8)->&'a u8{let q:for<'b>fn(&'b u8,&'a u8)->&'b u8=p;q(x,x)}
fn main() {}
