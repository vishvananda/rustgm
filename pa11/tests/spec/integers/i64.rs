fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:i64,y:i64)->i64 { x + y }
fn sub(x:i64,y:i64)->i64 { x - y }
fn mul(x:i64,y:i64)->i64 { x * y }
fn div(x:i64,y:i64)->i64 { x / y }
fn rem(x:i64,y:i64)->i64 { x % y }
fn bitand(x:i64,y:i64)->i64 { x & y }
fn bitor(x:i64,y:i64)->i64 { x | y }
fn bitxor(x:i64,y:i64)->i64 { x ^ y }
fn main() {
    check(add(9223372036854775804,3)==9223372036854775807);
    check(sub(9223372036854775807,3)==9223372036854775804);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(-9223372036854775808,0)==-9223372036854775808);
    check(!0i64 == -1);
    check(div(-79,11)== -7);
    check(rem(-79,11)== -2);
    check(mul(-7,-11)==77);
    check(sub(-9223372036854775805,3)==-9223372036854775808);
}
