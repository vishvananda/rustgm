fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(i16)]enum A{X=-2}const N:A=A::X;#[repr(u128)]enum E{A=N as u128,B}fn main(){check(E::A as u128==340282366920938463463374607431768211454 && E::B as u128==340282366920938463463374607431768211455);}
