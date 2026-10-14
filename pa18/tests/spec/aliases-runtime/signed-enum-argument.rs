fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(i8)]enum E{A=-3,B}type A<const N:usize>=[u8;N];fn main(){let x:A<{(E::B as isize+3) as usize}>=[17;1];check(x[0]==17);}
