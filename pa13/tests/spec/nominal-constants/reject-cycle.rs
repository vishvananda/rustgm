#[derive(PartialEq)]struct S(u8);const C:S=D;const D:S=C;fn f(x:S){match x{C=>(),_=>()}}
fn main() {}
