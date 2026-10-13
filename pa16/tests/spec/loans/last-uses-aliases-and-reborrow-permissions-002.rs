enum E<'a>{A(&'a mut u8),B}fn f<'a>(p:&'a mut u8)->E<'a>{E::A(p)}
fn main() {}
