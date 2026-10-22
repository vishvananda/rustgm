fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=F::B as isize}enum F{B=7}fn main(){check(E::A as isize==7);}