fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:i16,y:u128)->i16{x<<y} fn right(x:i16,y:i16)->i16{x>>y}
fn main() {
    check(left(1,14)==16384);
    check(left(1,15)==-32768);
    check(right(32767,14)==1);
    check(right(-17,2)== -5);
}
