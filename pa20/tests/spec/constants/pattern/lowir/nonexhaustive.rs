fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:bool=false;}impl A for u8{const C:bool=true;}fn f(x:bool){match x{<u8 as A>::C=>()}}fn main(){}