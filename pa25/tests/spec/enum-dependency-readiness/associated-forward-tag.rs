fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=S::N}struct S;impl S{const N:isize={let n=F::B as isize;n};}enum F{B=7}fn main(){check(E::A as isize==7);}