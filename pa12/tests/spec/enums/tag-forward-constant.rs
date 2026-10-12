fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
enum E{A=E::C as isize+1,B=E::A as isize+1,C=4}fn main(){check(E::A as u8==5 && E::B as u8==6 && E::C as u8==4);}
