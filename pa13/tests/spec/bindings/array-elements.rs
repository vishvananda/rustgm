fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let [a,b,c]=[2u64,3,5];check(a+b+c==10);}