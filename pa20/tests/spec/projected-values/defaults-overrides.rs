fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word{const VALUE:u8=5;}impl Word for u8{}impl Word for u16{const VALUE:u8=9;}trait Float{type Int:Word;}struct A;struct B;impl Float for A{type Int=u8;}impl Float for B{type Int=u16;}fn value<T:Float>()->u8{T::Int::VALUE}fn main(){check(value::<A>()==5);check(value::<B>()==9);}
