fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=7,B}type A<const N:usize>=[u8;N];fn main(){let x:A<{E::B as usize}>=[3;8];check(x[7]==3);}
