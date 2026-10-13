fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn f(x:N)->u8{match x{n if n.0==0=>n.0+1,n=>n.0+2}}fn main(){check(f(N(0))==1&&f(N(3))==5);}
