type F<'a>=for<'b>fn(&'a u8,&'b u8)->&'a u8;fn f<'a>(x:F<'a>)->F<'a>{x}
fn main() {}
