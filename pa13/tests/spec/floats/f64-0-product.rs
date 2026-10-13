fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;const NAN:f64=0.0/0.0;
fn f(x:(f64,bool))->u8{match x{(0.0,true)=>1,(..0.0,_)=>2,(_,false)=>3,_=>4}}fn main(){check(f((-0.0,true))==1&&f((-1.0,false))==2&&f((0.0/0.0,false))==3&&f((0.0/0.0,true))==4);}