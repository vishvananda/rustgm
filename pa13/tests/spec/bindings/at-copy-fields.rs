fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let n=N(3);let whole@N(x)=n;check(whole.0==3&&x==3);}