fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
struct S{a:u64,b:u64} fn main(){let mut s=S{a:1,b:2};let n=S{b:{s.a=9;5},..s};check(n.a==9 && n.b==5);}
