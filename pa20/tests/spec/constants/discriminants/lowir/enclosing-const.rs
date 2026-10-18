fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8=3;}#[repr(u8)]enum E<const N:u8>{A=S::<N>::C}fn main(){}