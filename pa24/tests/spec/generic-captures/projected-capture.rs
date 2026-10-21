fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick{type Out:Copy;}impl Pick for u8{type Out=u16;}fn take<T:Pick>(x:T::Out)->T::Out{let f=move||x;f()}fn main(){check(take::<u8>(900)==900);}
