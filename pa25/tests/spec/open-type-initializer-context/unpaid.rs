fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait V<'a>{const X:u8;}impl<'a,T:'a>V<'a>for T{const X:u8=7;}struct S<'a,T>(&'a u8,T);impl<'a,T:V<'a>>S<'a,T>{const C:u8={let n=T::X;n};}fn run<'a,'b>(_:&'a u8,_:&'b u8)->u8{S::<'b,&'a u8>::C}fn main(){let n=7;check(run(&n,&n)==7);}
