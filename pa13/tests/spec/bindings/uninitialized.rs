fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let (x,mut y):(u8,u8);x=1;y=2;y+=3;check(x==1&&y==5);}