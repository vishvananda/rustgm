fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{match 7isize{N=>F::B as isize,_=>0}}const N:isize=7;enum F{B=9}fn main(){check(E::A as isize==9);}