fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:i64,y:u128)->i64{x<<y} fn right(x:i64,y:i16)->i64{x>>y}
fn main() {
    check(left(1,62)==4611686018427387904);
    check(left(1,63)==-9223372036854775808);
    check(right(9223372036854775807,62)==1);
    check(right(-17,2)== -5);
}
