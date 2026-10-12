fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(u8)]enum E{A=10,B(),C{},D} fn main(){check(E::B() as u8==11 && E::C{} as u8==12 && E::D as u8==13);}
