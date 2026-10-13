fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f([_,tail@..]:[u64;3])->[u64;2]{tail}fn main(){let a=f([2,3,5]);check(a[0]==3&&a[1]==5);}