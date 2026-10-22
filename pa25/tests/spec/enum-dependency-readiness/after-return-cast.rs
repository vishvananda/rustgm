fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{return 1;let _=F::B as isize;0}enum F{B=E::A as isize}fn main(){check(E::A as isize==1);}