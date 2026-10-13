fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u64);struct S{a:N,b:N}fn f(S{a:N(x),b}:S)->u64{x+b.0}fn main(){check(f(S{a:N(7),b:N(9)})==16);}