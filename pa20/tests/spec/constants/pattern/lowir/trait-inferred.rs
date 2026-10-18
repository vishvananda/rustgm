fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:Self;}impl A for u8{const C:Self=7;}fn f(x:u8)->bool{match x{A::C=>true,_=>false}}fn main(){check(f(7));check(!f(9))}