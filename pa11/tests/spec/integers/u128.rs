fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:u128,y:u128)->u128 { x + y }
fn sub(x:u128,y:u128)->u128 { x - y }
fn mul(x:u128,y:u128)->u128 { x * y }
fn div(x:u128,y:u128)->u128 { x / y }
fn rem(x:u128,y:u128)->u128 { x % y }
fn bitand(x:u128,y:u128)->u128 { x & y }
fn bitor(x:u128,y:u128)->u128 { x | y }
fn bitxor(x:u128,y:u128)->u128 { x ^ y }
fn main() {
    check(add(340282366920938463463374607431768211452,3)==340282366920938463463374607431768211455);
    check(sub(340282366920938463463374607431768211455,3)==340282366920938463463374607431768211452);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(0,0)==0);
    check(!0u128 == 340282366920938463463374607431768211455);
}
