fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id(x:&S)->&S{x}fn identity<T>(x:T)->T{x}fn main(){let a=S(3);check(identity(id)(&a).0==3);}struct S(u8);
