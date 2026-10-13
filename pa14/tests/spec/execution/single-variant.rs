fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);enum E{A(N,N)}fn f(e:E)->u8{let E::A(a,_)=e;let E::A(_,b)=e;a.0+b.0}fn main(){check(f(E::A(N(2),N(7)))==9);}
