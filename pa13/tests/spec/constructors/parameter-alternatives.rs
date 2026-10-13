fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A(u8),B(u8)}fn f((E::A(n)|E::B(n)):E)->u8{n}fn main(){check(f(E::A(3))==3&&f(E::B(7))==7);}