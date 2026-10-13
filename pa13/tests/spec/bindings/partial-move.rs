fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let p=(N(3),N(7));let(a,_)=p;let b=p.1;check(a.0==3&&b.0==7);}