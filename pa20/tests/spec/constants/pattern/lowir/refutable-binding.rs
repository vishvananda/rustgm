fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:u8=7;}impl A for u8{}fn main(){let <u8 as A>::C=7u8;}