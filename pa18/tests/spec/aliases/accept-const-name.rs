const C:usize=3;type A<const N:usize>=[u8;N];fn f(x:A<{C}>)->[u8;3]{x}
fn main(){}
