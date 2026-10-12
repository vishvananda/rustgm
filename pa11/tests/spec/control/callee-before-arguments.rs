fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn a(x:u64)->u64{x+1} fn b(x:u64)->u64{x+20} fn entry()->u64{let mut f:fn(u64)->u64=a;f({f=b;1})}
fn main() {
    check(entry() == 2u64);
}
