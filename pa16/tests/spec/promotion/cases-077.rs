fn f<'a>(p:fn(&'a u8))->&'static [fn(&'static u8);0]{&[p;0]}
fn main() {}
