fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);struct S{a:N,b:N}fn main(){let xs=[S{a:N(2),b:N(3)}];let[S{a,..}]=xs;let[S{b,..}]=xs;check(a.0==2&&b.0==3);}