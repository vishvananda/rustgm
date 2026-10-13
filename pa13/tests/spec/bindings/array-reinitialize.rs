fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let mut xs=[N(2),N(3)];let[a,_]=xs;xs=[N(5),N(7)];let[b,c]=xs;check(a.0==2&&b.0==5&&c.0==7);}