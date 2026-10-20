trait B {}
impl<T> B for T {}
impl B for (u8, [u8]) {}
fn main() {}
