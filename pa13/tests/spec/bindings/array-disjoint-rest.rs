fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let xs=[N(2),N(3),N(5)];let[a,..]=xs;let[_,tail@..]=xs;check(a.0==2&&tail[0].0==3&&tail[1].0==5);}