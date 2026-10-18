fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:usize;}struct S;impl<'a>A for S{const C:usize=3;}fn main(){check(<S as A>::C==3);check(<S as A>::C==3);}