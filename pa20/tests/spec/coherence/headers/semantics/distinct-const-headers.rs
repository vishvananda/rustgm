fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<const N:usize>{type Out;const C:Self::Out;}impl A<2>for u8{type Out=u16;const C:u16=7;}impl A<3>for u8{type Out=u32;const C:u32=9;}fn main(){check(<u8 as A<2>>::C==7&&<u8 as A<3>>::C==9)}
