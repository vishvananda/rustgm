type A<const N:usize>=[u8;N];fn f(x:A<{C}>)->[u8;3]{x}const C:usize=D;const D:usize=3;
fn main(){}
