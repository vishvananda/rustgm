fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:u16,y:u128)->u16{x<<y} fn right(x:u16,y:i16)->u16{x>>y}
fn main() {
    check(left(1,14)==16384);
    check(left(1,15)==32768);
    check(right(65535,14)==3);
}
