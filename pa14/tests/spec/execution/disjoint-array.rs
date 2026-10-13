fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn f(a:[N;2])->u8{let[first,..]=a;let[..,last]=a;first.0+last.0}fn main(){check(f([N(2),N(7)])==9);}
