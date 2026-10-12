fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn value(b:bool)->i32{let x;if b{x=41;}else{x=3;}x}fn main() {
    check(value(true)==41);
    check(value(false)==3);
}
