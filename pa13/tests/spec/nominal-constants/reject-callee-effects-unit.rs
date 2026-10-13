#[derive(PartialEq)]struct S();fn effect(){}const C:S=({effect();S})();fn f(x:S){match x{C=>()}}
fn main() {}
