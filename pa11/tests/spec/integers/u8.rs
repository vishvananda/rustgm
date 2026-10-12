fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:u8,y:u8)->u8 { x + y }
fn sub(x:u8,y:u8)->u8 { x - y }
fn mul(x:u8,y:u8)->u8 { x * y }
fn div(x:u8,y:u8)->u8 { x / y }
fn rem(x:u8,y:u8)->u8 { x % y }
fn bitand(x:u8,y:u8)->u8 { x & y }
fn bitor(x:u8,y:u8)->u8 { x | y }
fn bitxor(x:u8,y:u8)->u8 { x ^ y }
fn main() {
    check(add(252,3)==255);
    check(sub(255,3)==252);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(0,0)==0);
    check(!0u8 == 255);
}
