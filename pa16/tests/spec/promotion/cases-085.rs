fn f<'a>(p:fn(&'static u8))->[fn(&'a u8);0]{[p;0]}
fn main() {}
