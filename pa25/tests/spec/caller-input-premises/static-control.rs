fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait V<'a>{const X:u8;}impl<'a,T:'a>V<'a>for T{const X:u8=7;}struct S<'a,T>(&'a u8,T);impl<'a,T:V<'a>>S<'a,T>{const C:u8={let n=T::X;n};const D:u8={let n=Self::C;n};}fn run<'b>(_:&'b u8)->u8{S::<'b,&'static u8>::C}fn main(){let n=7;check(run(&n)==7);}
