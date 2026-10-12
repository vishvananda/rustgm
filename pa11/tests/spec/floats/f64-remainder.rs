fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn rem(x:f64,y:f64)->f64{x%y}
fn main() {
    check(rem(7.5,2.0)==1.5);
    check(rem(-7.5,2.0)== -1.5);
    check(rem(7.5,-2.0)==1.5);
    check(rem(2.0,7.0)==2.0);
    check(rem(2.0,1.0/0.0)==2.0);
    check(rem(0.0,2.0)==0.0);
    check(1.0/rem(-4.0,2.0)== -1.0/0.0);
    check(rem(2.0,0.0)!=rem(2.0,0.0));
    check(rem(1.0/0.0,2.0)!=rem(1.0/0.0,2.0));
    check(rem(2.0,0.0/0.0)!=rem(2.0,0.0/0.0));
}
