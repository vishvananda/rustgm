fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
struct S{a:u64,b:u64}fn main(){let s=S{a:1,b:2};let t=S{a:7,..s};check(s.a==1 && s.b==2 && t.a==7 && t.b==2);}
