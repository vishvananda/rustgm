fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait V{type Out;const X:Self::Out;}impl<'a>V for &'a u8{type Out=u8;const X:u8=7;}struct S<T>(T);impl<T:V>S<T>{const C:T::Out={let n=T::X;n};}fn run<'a>(_:&'a u8)->u8{S::<&'a u8>::C}fn main(){let n=7;check(run(&n)==7);}
