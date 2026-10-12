fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn entry()->u64{let mut x=0;'a:loop{x+=1;loop{if x<3{continue 'a;}break 'a x;}}}
fn main() {
    check(entry() == 3u64);
}
