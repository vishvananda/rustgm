fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}impl<const N:usize>A for [u8;N]{fn f(self)->u8{self[0]}}fn main(){let x=<[u8;2]>::f([7,9]);check(x==7);}