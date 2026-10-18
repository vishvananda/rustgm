fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:isize=if true{3}else{1/0};}impl A for u8{}enum E{A=<u8 as A>::C,B}fn main(){check(E::B as isize==4);}