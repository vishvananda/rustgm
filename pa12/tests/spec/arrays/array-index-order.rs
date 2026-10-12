fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn main(){let mut n=0;let mut a=[1u8,2];a[{n+=1;if n==11{0}else{1}}]={n+=10;7};check(n==11 && a[0]==7 && a[1]==2);}
