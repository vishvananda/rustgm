fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A,B,C}fn f(e:E)->u8{match e{E::A=>1,E::B=>2,E::C=>3}}fn main(){check(f(E::A)==1&&f(E::B)==2&&f(E::C)==3);}