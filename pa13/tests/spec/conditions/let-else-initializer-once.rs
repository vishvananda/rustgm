fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let mut calls=0;let (1,n)=({calls+=1;(calls,7)}) else{return;};check(calls==1&&n==7);}