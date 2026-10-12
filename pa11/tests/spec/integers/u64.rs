fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:u64,y:u64)->u64 { x + y }
fn sub(x:u64,y:u64)->u64 { x - y }
fn mul(x:u64,y:u64)->u64 { x * y }
fn div(x:u64,y:u64)->u64 { x / y }
fn rem(x:u64,y:u64)->u64 { x % y }
fn bitand(x:u64,y:u64)->u64 { x & y }
fn bitor(x:u64,y:u64)->u64 { x | y }
fn bitxor(x:u64,y:u64)->u64 { x ^ y }
fn main() {
    check(add(18446744073709551612,3)==18446744073709551615);
    check(sub(18446744073709551615,3)==18446744073709551612);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(0,0)==0);
    check(!0u64 == 18446744073709551615);
}
