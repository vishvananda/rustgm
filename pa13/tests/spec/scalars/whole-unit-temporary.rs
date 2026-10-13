fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N;fn main(){let whole@N=N;let _=whole;check(true);}