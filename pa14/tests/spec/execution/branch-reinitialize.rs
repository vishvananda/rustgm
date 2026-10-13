fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn take(x:N)->u8{x.0}fn f(c:bool)->u8{let mut x=N(3);let mut n=0;if c{n=take(x);x=N(7);}n+take(x)}fn main(){check(f(true)==10&&f(false)==3);}
