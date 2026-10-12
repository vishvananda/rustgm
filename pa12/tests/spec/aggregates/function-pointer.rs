fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
struct S{a:u64,b:u8} fn id(x:S)->S{x} fn main(){let f:fn(S)->S=id;let s=f(S{a:44,b:3});check(s.a==44 && s.b==3);}
