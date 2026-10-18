fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;const C:Self::Out;}impl A for u8{type Out=usize;const C:usize=3;}fn main(){check(<u8 as A>::C==3);}