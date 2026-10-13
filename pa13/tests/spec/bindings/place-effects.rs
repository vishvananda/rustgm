fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let a=[(3u8,4u16)];let mut i=0;let (x,y)=a[{i+=1;0}];check(i==1&&x==3&&y==4);}