fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn calc(x:f64,y:f64)->f64{(x+y)*(x-y)/y}
fn main() {
    check(calc(7.5,2.5)==20.0);
}
