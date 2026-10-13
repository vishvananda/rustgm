fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
#[repr(C,u8)]enum E{A(u64,u8)}fn main(){let E::A(a,b)=E::A(13,2);check(a==13&&b==2);}