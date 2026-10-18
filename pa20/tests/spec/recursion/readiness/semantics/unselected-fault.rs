#![allow(unconditional_panic)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;}impl A for u8{const C:usize=3;}impl A for u16{const C:usize=1/0;}