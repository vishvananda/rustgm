fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:u8=255;}impl A for u8{}#[repr(u8)]enum E{A=<u8 as A>::C,B}fn main(){}