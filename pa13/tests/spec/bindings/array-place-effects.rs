fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let xs=[[1u8,2]];let mut n=0;let[a,b]=xs[{n+=1;0}];check(a==1&&b==2&&n==1);}