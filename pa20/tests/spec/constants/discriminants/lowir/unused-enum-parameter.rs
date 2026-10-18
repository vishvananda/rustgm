fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{const C:u8=3;}#[repr(u8)]enum E<T>{A(T)=S::<u8>::C}fn main(){let x=E::A(7u8);match x{E::A(v)=>check(v==7)}}