fn f<'a>(g:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{g(x)}
fn main() {}
