fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
fn f(x:f64)->u8{let mut n=0;match x{-2.0..=2.0|0.0..=4.0|_ if {n+=1;false}=>(),_=>()}n}fn main(){check(f(1.0)==3&&f(-1.0)==2&&f(0.0/0.0)==1);}