#[derive(PartialEq)]struct S(u8);struct T(u8);const C:S=S(7);fn f(x:T){match x{C=>(),_=>()}}
fn main() {}
