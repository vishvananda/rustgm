fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a,'b>{type Out:'b;}fn keep<'a,T:'a>(_:T){}fn wrong<'a,'b,T:A<'a,'b>>(x:T::Out){keep::<'a,_>(x)}fn main(){}