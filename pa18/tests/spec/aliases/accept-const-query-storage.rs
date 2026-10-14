type A<const N:usize>=[u8;N];fn f(_:A<{1+2}>,_:A<{3+2}>,_:A<{C}>,_:A<{D}>){}const C:usize=7;const D:usize=9;
fn main(){}
