fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let mut n=0;let x=match {n+=1;(n,7)}{(1,v)=>v,_=>9};check(n==1&&x==7);}