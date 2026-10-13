fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let [a,mut b]:[u8;2];a=3;b=7;b+=2;check(a==3&&b==9);}