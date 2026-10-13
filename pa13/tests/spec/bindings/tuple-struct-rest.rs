fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct S(u8,u16,u32,u64);fn main(){let S(a,..,d)=S(1,2,3,4);check(a==1&&d==4);}