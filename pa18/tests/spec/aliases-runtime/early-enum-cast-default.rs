fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type I<T>=T;enum E{A=C as I<isize>,B}const C:u8=7;type A<const N:usize={E::B as usize}>=[u8;N];fn read(x:A)->u8{x[7]}fn main(){check(read([13;8])==13);}
