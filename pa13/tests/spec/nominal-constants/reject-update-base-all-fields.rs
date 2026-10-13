#[derive(PartialEq)]struct S{a:u8}fn base()->S{S{a:0}}const C:S=S{a:7,..base()};fn f(x:S){match x{C=>(),_=>()}}
fn main() {}
