fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{V(u8)}type A=E;impl E{fn V()->u8{9}}fn main(){let E::V(x)=A::V(7);check(x==7);}