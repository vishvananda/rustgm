struct X;#[derive(PartialEq)]enum E{A,B(X)}const C:E=E::A;fn f(x:E){match x{C=>(),_=>()}}
fn main() {}
