fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn unit()->(){} fn main(){let x=unit();let y=x;let _=y;check(true);}
