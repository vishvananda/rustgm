fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let ((x,y)|(y,x))=(1u8,2u8);check(x==1&&y==2);}