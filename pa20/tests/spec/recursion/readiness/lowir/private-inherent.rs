fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X<const N:usize>([u8;N]);mod m{pub struct S;impl S{const C:usize=3;}}fn main(){let _x:X<{m::S::C}>=X([7;3]);}