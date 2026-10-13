fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
enum E{A(f64),B}fn f(x:E)->u8{match x{E::A(..0.0)=>1,E::A(0.0..)=>2,E::A(_)=>3,E::B=>4}}fn main(){check(f(E::A(-1.0))==1&&f(E::A(-0.0))==2&&f(E::A(0.0/0.0))==3&&f(E::B)==4);}