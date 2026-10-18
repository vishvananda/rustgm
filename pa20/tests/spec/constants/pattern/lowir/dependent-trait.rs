fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:u8;}fn f<T:A>(x:u8)->bool{match x{T::C=>true,_=>false}}fn main(){}