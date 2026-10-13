enum E<'a>{A(&'a u8),B}fn f<'a>(x:&'a u8)->E<'a>{E::A(x)}
fn main() {}
