fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A(u8),B}fn f()->u8{if let E::A(n)=(return 7){n}else{9}}fn main(){check(f()==7);}