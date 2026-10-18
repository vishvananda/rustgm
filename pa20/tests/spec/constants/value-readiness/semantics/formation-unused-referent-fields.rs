fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C.1}>{X([7;3])}enum O<T>{None,Some(T)}struct S{a:X<{<u8 as A>::C.1}>}trait A{const C:(O<&'static S>,usize);}impl A for u8{const C:(O<&'static S>,usize)=(O::None,3);}fn main(){check(f().0[2]==7);}
