fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn main(){let mut sum=0;let mut n=0;while n<4{let x;n+=1;x=n;sum+=x;}check(sum==10);}
