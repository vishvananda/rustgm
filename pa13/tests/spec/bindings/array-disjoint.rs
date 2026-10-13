fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let xs=[N(2),N(3)];let[a,_]=xs;let[_,b]=xs;check(a.0==2&&b.0==3);}