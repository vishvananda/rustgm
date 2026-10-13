fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A(u8)}fn main(){let n=17;let out=if let E::A(n)=E::A(3)&&let E::A(n)=E::A(n+1)&&n==4{n}else{n};check(out==4&&n==17);}