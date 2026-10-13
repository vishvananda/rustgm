fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn take(x:N)->u8{x.0}fn main(){let x=N(7);let v=match x{n if n.0==0=>take(n),_=>take(x)};check(v==7);}