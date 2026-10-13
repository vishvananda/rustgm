fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const NEG:f64=-0.0;const POS:f64=0.0;fn f(x:f64)->u8{let mut n=0;match x{0.0|-0.0 if {n+=1;false}=>(),_=>()}n}
fn main(){check(f(-0.0)==2&&f(0.0)==2&&f(0.0/0.0)==0);}