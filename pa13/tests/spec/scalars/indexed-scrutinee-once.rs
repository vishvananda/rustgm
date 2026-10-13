fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let mut n=0;let xs=[7u8];let x=match xs[{n+=1;0}]{7=>3,_=>5};check(n==1&&x==3);}