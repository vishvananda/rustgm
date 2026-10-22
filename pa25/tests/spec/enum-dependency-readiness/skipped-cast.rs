fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{if false{F::B as isize}else{1}}enum F{B=7}fn main(){check(E::A as isize==1&&F::B as isize==7);}