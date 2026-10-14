type A<const N:usize>=[u8;N];fn f(x:A<{1+2}>)->[u8;3]{x}
fn main(){}
