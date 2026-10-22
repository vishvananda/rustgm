fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f(),B=f()+1}const fn f()->isize{F::B as isize}enum F{B=7}fn main(){check(E::A as isize==7&&E::B as isize==8);}