fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<T>{const C:usize;}struct S;impl<T:'static>A<T>for S{const C:usize=3;}fn main(){check(<S as A<fn(&u8)->&u8>>::C==3);}