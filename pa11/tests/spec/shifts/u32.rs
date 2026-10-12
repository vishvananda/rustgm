fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn left(x:u32,y:u128)->u32{x<<y} fn right(x:u32,y:i16)->u32{x>>y}
fn main() {
    check(left(1,30)==1073741824);
    check(left(1,31)==2147483648);
    check(right(4294967295,30)==3);
}
