struct S<'a>(&'a u8);fn f<'a>(x:&'a u8)->S<'a>{S(x)}
fn main() {}
