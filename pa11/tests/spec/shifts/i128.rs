fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:i128,y:u128)->i128{x<<y} fn right(x:i128,y:i16)->i128{x>>y}
fn main() {
    check(left(1,126)==85070591730234615865843651857942052864);
    check(left(1,127)==-170141183460469231731687303715884105728);
    check(right(170141183460469231731687303715884105727,126)==1);
    check(right(-17,2)== -5);
}
