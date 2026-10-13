fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A(u8),B}fn f(x:E)->u8{let E::A(n)=x else{return 99;};n}fn main(){check(f(E::A(7))==7&&f(E::B)==99);}