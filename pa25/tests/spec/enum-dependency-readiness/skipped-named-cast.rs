fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const V:F=F::B;enum E{A=f()}const fn f()->isize{if false{V as isize}else{1}}enum F{B=E::A as isize}fn main(){check(E::A as isize==1);}