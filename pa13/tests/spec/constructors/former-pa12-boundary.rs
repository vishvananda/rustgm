fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A,B}fn main(){let n=match E::A{E::A=>1,E::B=>2};}