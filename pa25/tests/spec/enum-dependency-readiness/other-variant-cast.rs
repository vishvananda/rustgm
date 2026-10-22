fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{F::B as isize}enum F{B=7,C=E::A as isize+1}fn main(){check(E::A as isize==7&&F::C as isize==8);}