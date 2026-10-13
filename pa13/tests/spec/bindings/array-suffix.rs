fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let [a,tail@..]=[2u8,3,5];check(a==2&&tail[0]==3&&tail[1]==5);}