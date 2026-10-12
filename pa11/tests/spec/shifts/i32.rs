fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:i32,y:u128)->i32{x<<y} fn right(x:i32,y:i16)->i32{x>>y}
fn main() {
    check(left(1,30)==1073741824);
    check(left(1,31)==-2147483648);
    check(right(2147483647,30)==1);
    check(right(-17,2)== -5);
}
