fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
#[repr(i8)]enum E{A=-7,B=Self::A as i8+10,C}fn main(){check(E::A as i8 == -7 && E::B as i8==3 && E::C as i8==4);}
