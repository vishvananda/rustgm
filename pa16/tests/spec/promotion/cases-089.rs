fn give_static(x:&'static u8)->&'static u8{x}fn f<'a>(p:&'a u8)->[&'static u8;0]{[give_static(p);0]}
fn main() {}
