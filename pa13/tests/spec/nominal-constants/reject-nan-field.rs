#[derive(PartialEq)]struct S(f32);const C:S=S(0.0/0.0);fn f(x:S){match x{C=>(),_=>()}}
fn main() {}
