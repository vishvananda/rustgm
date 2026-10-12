fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn r32(x:f32,y:f32)->f32{x%y} fn r64(x:f64,y:f64)->f64{x%y}
fn main() {
    check(r32(4.203895392974451e-45,2.802596928649634e-45)==1.401298464324817e-45);
    check(r64(1.5e-323,1.0e-323)==5e-324);
    check(r64(1.0e308,3.0)==2.0);
    check(r32(3.4028234663852886e38,3.0)==0.0);
}
