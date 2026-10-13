enum E<'a>{A(&'a u8),B}fn f<'a>()->E<'a>{E::<'a>::B}
fn main() {}
