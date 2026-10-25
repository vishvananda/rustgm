#![allow(dead_code,unused_variables,unreachable_code)]
trait Plain{}trait Has{type Out:Plain;fn make()->Self::Out;}fn probe<T:Has>(){T::make();}
fn main(){}
