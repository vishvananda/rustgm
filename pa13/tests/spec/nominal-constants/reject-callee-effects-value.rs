#[derive(PartialEq)]struct S(u8);fn effect(){}const C:S=({effect();S})(7);fn f(x:S){match x{C=>(),_=>()}}
fn main() {}
