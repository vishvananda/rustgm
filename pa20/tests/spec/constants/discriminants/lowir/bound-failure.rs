fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:isize=3;}struct S<T>(T);struct U;impl<T:Copy>A for S<T>{}enum E{A=<S<U>as A>::C}fn main(){}