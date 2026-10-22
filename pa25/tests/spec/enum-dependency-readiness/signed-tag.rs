fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{F::B as isize}#[repr(i8)]enum F{B=-3}fn main(){check(E::A as isize== -3);}