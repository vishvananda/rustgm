fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
#[repr(transparent)]struct S((),u128,());fn f(s:S)->S{s}fn main(){let x=f(S((),340282366920938463463374607431768211455,()));check(x.1==340282366920938463463374607431768211455);}
