fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
const N:u8=BASE+1;const BASE:u8=4;#[repr(u8)]enum E{A=N*2,B}fn main(){check(E::A as u8==10 && E::B as u8==11);}
