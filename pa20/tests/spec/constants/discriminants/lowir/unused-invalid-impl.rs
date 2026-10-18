fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:isize=3;}trait B{fn f();}impl A for u8{}impl B for u8{}enum E{A=<u8 as A>::C}fn main(){}