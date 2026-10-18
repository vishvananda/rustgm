fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:usize;}impl<const N:usize>A for u8{const C:usize=N;}fn main(){}
