struct S<'a>{value:&'a u8}fn f<'a>(x:&'a u8)->S<'a>{S::<'a>{value:x}}
fn main() {}
