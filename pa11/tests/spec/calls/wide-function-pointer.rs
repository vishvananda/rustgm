fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn add(x:u128,y:u128)->u128{x+y} fn call(f:fn(u128,u128)->u128)->u128{f(18446744073709551616,9)}
fn main() {
    check(call(add)==18446744073709551625);
}
