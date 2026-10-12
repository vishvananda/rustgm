fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
enum E{A=42} fn main(){check(E::A as u8==42);}
