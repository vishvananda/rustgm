fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:u16,y:u16)->u16 { x + y }
fn sub(x:u16,y:u16)->u16 { x - y }
fn mul(x:u16,y:u16)->u16 { x * y }
fn div(x:u16,y:u16)->u16 { x / y }
fn rem(x:u16,y:u16)->u16 { x % y }
fn bitand(x:u16,y:u16)->u16 { x & y }
fn bitor(x:u16,y:u16)->u16 { x | y }
fn bitxor(x:u16,y:u16)->u16 { x ^ y }
fn main() {
    check(add(65532,3)==65535);
    check(sub(65535,3)==65532);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(0,0)==0);
    check(!0u16 == 65535);
}
