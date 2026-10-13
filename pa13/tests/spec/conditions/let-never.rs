fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A(u8),B}fn f()->u8{let E::A(n)=return 7 else{return 9;};n}fn main(){check(f()==7);}