fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let ([x,y]|[y,x])=[2u8,3];check(x==2&&y==3);}