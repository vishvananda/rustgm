fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
fn f(x:f64)->u8{let -0.0..=0.0=x else{return 2;};1}fn main(){check(f(0.0)==1&&f(-0.0)==1&&f(0.0/0.0)==2);}