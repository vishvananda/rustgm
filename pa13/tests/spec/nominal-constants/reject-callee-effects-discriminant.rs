enum E{A()}fn effect(){}const C:isize=({effect();E::A})() as isize;fn f(x:isize){match x{C=>(),_=>()}}
fn main() {}
