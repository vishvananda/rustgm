fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let x=N(7);let v=match x{n if match n{N(v) if v>0=>true,_=>false}=>n.0,_=>99};check(v==7);}