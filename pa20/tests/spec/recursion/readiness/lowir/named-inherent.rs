fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X<const N:usize>([u8;N]);struct S;fn main(){let x:X<{S::C}>=X([7;3]);check(x.0[2]==7);}impl S{const C:usize=3;}