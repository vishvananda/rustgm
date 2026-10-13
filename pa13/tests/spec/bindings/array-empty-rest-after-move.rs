fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let xs=[N(2)];let[a]=xs;let[_,tail@..]=xs;let _=tail;check(a.0==2);}