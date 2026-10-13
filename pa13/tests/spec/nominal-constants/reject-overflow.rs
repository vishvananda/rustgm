#[derive(PartialEq)]struct S(u8);const C:S=S(255+1);fn f(x:S){match x{C=>(),_=>()}}
fn main() {}
