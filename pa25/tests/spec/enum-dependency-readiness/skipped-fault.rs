fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{if false{1/0}else{1}}fn main(){check(E::A as isize==1);}