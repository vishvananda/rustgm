fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{u8::C}>=X([7;3]);check(x.0[2]==7);}