fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:i16,y:i16)->i16 { x + y }
fn sub(x:i16,y:i16)->i16 { x - y }
fn mul(x:i16,y:i16)->i16 { x * y }
fn div(x:i16,y:i16)->i16 { x / y }
fn rem(x:i16,y:i16)->i16 { x % y }
fn bitand(x:i16,y:i16)->i16 { x & y }
fn bitor(x:i16,y:i16)->i16 { x | y }
fn bitxor(x:i16,y:i16)->i16 { x ^ y }
fn main() {
    check(add(32764,3)==32767);
    check(sub(32767,3)==32764);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(-32768,0)==-32768);
    check(!0i16 == -1);
    check(div(-79,11)== -7);
    check(rem(-79,11)== -2);
    check(mul(-7,-11)==77);
    check(sub(-32765,3)==-32768);
}
