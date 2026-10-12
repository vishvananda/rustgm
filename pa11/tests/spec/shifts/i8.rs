fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:i8,y:u128)->i8{x<<y} fn right(x:i8,y:i16)->i8{x>>y}
fn main() {
    check(left(1,6)==64);
    check(left(1,7)==-128);
    check(right(127,6)==1);
    check(right(-17,2)== -5);
}
