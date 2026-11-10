fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const VALUE:u8;}trait B{const VALUE:u8;}trait Float{type Int:A+B;}fn value<T:Float>()->u8{T::Int::VALUE}fn main(){}
