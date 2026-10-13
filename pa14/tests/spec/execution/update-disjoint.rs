fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);struct S{a:N,b:N}fn f(x:S)->u8{let a=x.a;let y=S{a:N(8),..x};a.0+y.a.0+y.b.0}fn main(){check(f(S{a:N(2),b:N(7)})==17);}
