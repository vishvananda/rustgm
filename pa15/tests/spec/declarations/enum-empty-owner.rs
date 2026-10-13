enum E<'a>{A(&'a u8)}fn f<'a>(x:&'a u8)->E<'a>{E::<>::A::<'a>(x)}
fn main() {}
