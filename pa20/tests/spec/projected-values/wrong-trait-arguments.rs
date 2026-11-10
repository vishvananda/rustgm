fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word<T,const N:usize>{const VALUE:T;}trait Float{type Int:Word<u8,3>;}fn value<T:Float>()->u16{<T::Int as Word<u16,5>>::VALUE}fn main(){}
