fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:u8,y:u128)->u8{x<<y} fn right(x:u8,y:i16)->u8{x>>y}
fn main() {
    check(left(1,6)==64);
    check(left(1,7)==128);
    check(right(255,6)==3);
}
