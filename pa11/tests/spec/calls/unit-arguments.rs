fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn select(_:(),x:i32,_:(),y:i32)->i32{x*10+y}
fn main(){let mut x=1;let result=select({x=2;},x,{x=3;},x);check(result==23);check(x==3);}
