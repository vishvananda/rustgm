fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let [a,..,b]=[2u8,3,5,7];check(a==2&&b==7);}