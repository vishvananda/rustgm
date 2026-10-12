fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let mut x=0;(true&&{x+=1;false})||{x+=2;true};x}
fn main() {
    check(entry() == 3u64);
}
