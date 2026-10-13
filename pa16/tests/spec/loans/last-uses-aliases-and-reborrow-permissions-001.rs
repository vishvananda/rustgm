struct S<'a>(&'a u8);fn f<'a>(p:&'a u8)->S<'a>{S(p)}
fn main() {}
