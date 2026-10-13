fn take(_: [&'static u8;0]){}fn f<'a>(p:&'a u8){let a=[p;0];take(a)}
fn main() {}
