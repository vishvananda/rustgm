fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N;fn take(_:N)->u8{7}fn main(){let v=match N{n if true=>take(n),_=>99};check(v==7);}