fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let ([a@..,_]|[_,a@..])=[2u8,3];check(a[0]==2);}