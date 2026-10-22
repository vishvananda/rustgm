fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{F::B as isize}enum F{B=g()}const fn g()->isize{E::A as isize}fn main(){}