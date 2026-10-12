fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
struct N(u64); fn f()->N{N(3)} fn main(){let a=[f();0];let b=[f();1];check(b[0].0==3);let _=a;}
