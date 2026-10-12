fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn sum(x0:f64,x1:f64,x2:f64,x3:f64,x4:f64,x5:f64,x6:f64,x7:f64,x8:f64,x9:f64,x10:f64,x11:f64)->f64{x0+x1+x2+x3+x4+x5+x6+x7+x8+x9+x10+x11} fn main(){check(sum(0.0,1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0)==66.0);}
