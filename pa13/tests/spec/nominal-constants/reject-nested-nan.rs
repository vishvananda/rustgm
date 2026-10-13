#[derive(PartialEq)]struct S([f64;2]);const C:S=S([1.0,0.0/0.0]);fn f(x:S){match x{C=>(),_=>()}}
fn main() {}
