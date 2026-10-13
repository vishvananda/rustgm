fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum V{}enum E{A(V),B}fn f(e:E)->u8{match e{E::B=>7}}fn main(){check(f(E::B)==7);}