#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn take(x:S)->bool{true}fn f(x:S){match x{v@C if take(v)=>(),_=>()}}
fn main() {}
