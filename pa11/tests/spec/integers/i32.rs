fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:i32,y:i32)->i32 { x + y }
fn sub(x:i32,y:i32)->i32 { x - y }
fn mul(x:i32,y:i32)->i32 { x * y }
fn div(x:i32,y:i32)->i32 { x / y }
fn rem(x:i32,y:i32)->i32 { x % y }
fn bitand(x:i32,y:i32)->i32 { x & y }
fn bitor(x:i32,y:i32)->i32 { x | y }
fn bitxor(x:i32,y:i32)->i32 { x ^ y }
fn main() {
    check(add(2147483644,3)==2147483647);
    check(sub(2147483647,3)==2147483644);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(-2147483648,0)==-2147483648);
    check(!0i32 == -1);
    check(div(-79,11)== -7);
    check(rem(-79,11)== -2);
    check(mul(-7,-11)==77);
    check(sub(-2147483645,3)==-2147483648);
}
