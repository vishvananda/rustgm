fn f<'a>(p:&'a u8)->&'static [&'static u8;0]{let q=[p;0];&q}
fn main() {}
