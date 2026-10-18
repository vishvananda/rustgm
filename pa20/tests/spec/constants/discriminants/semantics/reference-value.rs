fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:&'static isize=&3;}impl A for u8{}enum E{A=*<u8 as A>::C,B}fn main(){check(E::B as isize==4);}