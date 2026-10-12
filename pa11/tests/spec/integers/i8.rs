fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:i8,y:i8)->i8 { x + y }
fn sub(x:i8,y:i8)->i8 { x - y }
fn mul(x:i8,y:i8)->i8 { x * y }
fn div(x:i8,y:i8)->i8 { x / y }
fn rem(x:i8,y:i8)->i8 { x % y }
fn bitand(x:i8,y:i8)->i8 { x & y }
fn bitor(x:i8,y:i8)->i8 { x | y }
fn bitxor(x:i8,y:i8)->i8 { x ^ y }
fn main() {
    check(add(124,3)==127);
    check(sub(127,3)==124);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(-128,0)==-128);
    check(!0i8 == -1);
    check(div(-79,11)== -7);
    check(rem(-79,11)== -2);
    check(mul(-7,-11)==77);
    check(sub(-125,3)==-128);
}
