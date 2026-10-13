fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct S(u8,u64);fn main(){let S{1:b,0:a}=S(3,7);check(a==3&&b==7);}