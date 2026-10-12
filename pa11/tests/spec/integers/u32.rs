fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:u32,y:u32)->u32 { x + y }
fn sub(x:u32,y:u32)->u32 { x - y }
fn mul(x:u32,y:u32)->u32 { x * y }
fn div(x:u32,y:u32)->u32 { x / y }
fn rem(x:u32,y:u32)->u32 { x % y }
fn bitand(x:u32,y:u32)->u32 { x & y }
fn bitor(x:u32,y:u32)->u32 { x | y }
fn bitxor(x:u32,y:u32)->u32 { x ^ y }
fn main() {
    check(add(4294967292,3)==4294967295);
    check(sub(4294967295,3)==4294967292);
    check(mul(7,11)==77);
    check(div(77,11)==7);
    check(rem(79,11)==2);
    check(bitand(10,12)==8);
    check(bitor(10,12)==14);
    check(bitxor(10,12)==6);
    check(add(0,0)==0);
    check(!0u32 == 4294967295);
}
