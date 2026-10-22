fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{F::C as isize}enum F{B=7,C}fn main(){check(E::A as isize==8);}