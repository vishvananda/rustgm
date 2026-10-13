fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);struct S{a:N,b:N}fn take(x:N)->u8{x.0}fn f(mut s:S)->u8{let n=take(s.a);s.a=N(n+1);take(s.a)+take(s.b)}fn main(){check(f(S{a:N(2),b:N(7)})==10);}
