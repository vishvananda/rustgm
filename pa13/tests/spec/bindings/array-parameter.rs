fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f([a,tail@..]:[u64;3])->u64{a+tail[0]*2+tail[1]*3}fn main(){check(f([2,3,5])==23);}