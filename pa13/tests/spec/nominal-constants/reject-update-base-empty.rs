#[derive(PartialEq)]struct S{}fn base()->S{S{}}const C:S=S{..base()};fn f(x:S){match x{C=>()}}
fn main() {}
