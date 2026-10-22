fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum F{B=7}const fn f()->isize{F::B as isize}const N:isize=f();fn main(){check(N==7);}