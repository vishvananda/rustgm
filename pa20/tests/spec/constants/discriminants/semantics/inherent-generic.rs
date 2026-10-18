fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{const C:isize=3;}enum E{A=S::<u8>::C,B}fn main(){check(E::B as isize==4);}